// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// use tauri::ipc::Response;
use serde::Serialize;
use serde::Deserialize;
use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use futures::TryStreamExt;
use std::env::Args;
use std::process::Command;
use std::process::ExitStatus;
use walkdir::WalkDir;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize, Deserialize)]
struct Order {
    order_number: String,
    part_number: String,
    due_quantity: f64,
    assn_number: String,
}

#[derive(Serialize)]
struct PrintOrder {
    order_number: String,
    part_number: String,
    due_quantity: f64,
    assn_number: String,
    print_type: String,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct PrintOrderRow {
    id: i32,
    print_type: String,
    notes: String,
}

#[tauri::command]
async fn get_order_number_info(order_number: String) -> Result<Vec<Order>, String> {

    let mut config = Config::new();
    config.host("PXSVSSQL01");
    config.port(1433); // default SQL Server port
    config.authentication(AuthMethod::sql_server("ReportS", "Report$"));
    config.database("ExactMAXPixus");
    config.trust_cert(); // optional if using self-signed SSL cert

    // Connect
    let tcp = TcpStream::connect(config.get_addr())
        .await
        .map_err(|e| format!("TCP connect error: {}", e))?;
    let tcp = tcp.compat();

    let mut client = Client::connect(config, tcp)
        .await
        .map_err(|e| format!("DB connect error: {}", e))?;


    let query =
    "SELECT DISTINCT om.ORDNUM_10, om.PRTNUM_10, om.DUEQTY_10, 
        CASE pm.TYPE_01 WHEN 'S'        THEN pm.PRTNUM_01
                                        ELSE om.PRTNUM_10
        END AS ASSPRT
FROM    Requirement_Detail rd, Order_Master om, Product_Structure ps, Part_Master pm
WHERE   om.ORDNUM_10 = @P1
        AND om.ORDNUM_10 = rd.ORDNUM_11
        AND om.DUEQTY_10 > 0 
        AND pm.PRTNUM_01 = ps.COMPRT_02
        AND ps.PARPRT_02 = om.PRTNUM_10
        AND om.STATUS_10 = '3'
        AND ((pm.TYPE_01 = 'S' AND (LEFT(pm.PRTNUM_01, 3) = '0' + '2' + 'A' OR LEFT(pm.PRTNUM_01, 4) = 'K' + '0' + '2' + 'A') )
            OR rd.PRTNUM_11 = ps.COMPRT_02)";


    let mut stream = client
        .query(query, &[&order_number])
        .await
        .map_err(|e| format!("Query error: {}", e))?;

    let mut orders = Vec::new();

    while let Some(item) = stream.try_next().await.map_err(|e| format!("Row error: {}", e))? {
        if let Some(row) = item.into_row() {
            let order_number: Option<&str> = row.get(0);
            let part_number: Option<&str> = row.get(1);
            let due_quantity: Option<f64> = row.get(2);
            let assn_number: Option<&str> = row.get(3);

            orders.push(Order {
                order_number: order_number.map(|s| s.to_string()).expect("ordernumber should have a value"),
                part_number: part_number.map(|s| s.to_string()).expect("part_number should have a value"),
                due_quantity: due_quantity.expect("due_quantity should have a value"),
                assn_number: assn_number.map(|s| s.to_string()).expect("assn_number should have a value"),
            });
        }
    }

    Ok(orders)
}

#[tauri::command]
async fn get_print_items(order_number: String) -> Result<Vec<PrintOrder>, String> {
    let mut config = Config::new();
    config.host("PXSVSSQL01");
    config.port(1433); // default SQL Server port
    config.authentication(AuthMethod::sql_server("ReportS", "Report$"));
    config.database("ExactMAXPixus");
    config.trust_cert(); // optional if using self-signed SSL cert

    // Connect
    let tcp = TcpStream::connect(config.get_addr())
        .await
        .map_err(|e| format!("TCP connect error: {}", e))?;
    let tcp = tcp.compat();

    let mut client = Client::connect(config, tcp)
        .await
        .map_err(|e| format!("DB connect error: {}", e))?;

    let query =
    "SELECT  om.ORDNUM_10, om.PRTNUM_10, om.DUEQTY_10, 
        CASE pm.TYPE_01 WHEN 'S'        THEN ps.COMPRT_02
                                        ELSE om.PRTNUM_10
        END AS ASSPRT,
rd.PRTNUM_11, wn.NOTES_61
FROM    Requirement_Detail rd, Windows_Notes wn, Order_Master om, Product_Structure ps, Part_Master pm
WHERE   om.ORDNUM_10 = @P1
        AND om.ORDNUM_10 = rd.ORDNUM_11
        AND wn.COMPRT_61 = rd.PRTNUM_11
        AND (LEFT(rd.PRTNUM_11, 3) = '94A'
            OR LEFT(rd.PRTNUM_11, 4) = 'K94A'
            OR rd.PRTNUM_11 = 'Initial DOCS'
            OR rd.PRTNUM_11 = 'Final DOCS')
        AND om.DUEQTY_10 > 0 
        AND pm.PRTNUM_01 = ps.COMPRT_02
        AND ps.PARPRT_02 = om.PRTNUM_10
        AND om.STATUS_10 = '3'
        AND ( (pm.TYPE_01 = 'S' AND wn.PRTNUM_61 = ps.COMPRT_02)
        OR (rd.PRTNUM_11 = ps.COMPRT_02 AND wn.PRTNUM_61 = om.PRTNUM_10))

ORDER BY wn.MAXID";


    let mut stream = client
        .query(query, &[&order_number])
        .await
        .map_err(|e| format!("Query error: {}", e))?;

    let mut orders = Vec::new();
    
    // add BOM SNL and config sheet
    orders.push(PrintOrder { // needs ASSN
        order_number: order_number.to_string(),
        part_number: "".to_string(),
        due_quantity: 1.0, 
        assn_number: "".to_string(),
        print_type: "BOM".to_string(),
        notes: Some("Bill of Materials".to_string()),
    });
    orders.push(PrintOrder { // needs PARTNUM for locating
        order_number: order_number.to_string(),
        part_number: "".to_string(),
        due_quantity: 1.0, 
        assn_number: "".to_string(),
        print_type: "Config".to_string(),
        notes: Some("Configuration Sheet".to_string()),
    });
    orders.push(PrintOrder { // needs ORDNUM
        order_number: order_number.to_string(),
        part_number: "".to_string(),
        due_quantity: 1.0, 
        assn_number: "".to_string(),
        print_type: "SNL".to_string(),
        notes: Some("Serial Number List".to_string()),
    });

    while let Some(item) = stream.try_next().await.map_err(|e| format!("Row error: {}", e))? {
        if let Some(row) = item.into_row() {
            let order_number: Option<&str> = row.get(0);
            let part_number: Option<&str> = row.get(1);
            let due_quantity: Option<f64> = row.get(2);
            let assn_number: Option<&str> = row.get(3);
            let print_type: Option<&str> = row.get(4);
            let notes: Option<&str> = row.get(5);
            // remove if note empty OR if starting with ~
            let c = notes.and_then(|s| s.chars().next());
            if notes == Some("") || c == Some('~') {
                continue;
            } else if c == Some('?') {
                //deal with starting ? 
            }

            orders.push(PrintOrder {
                order_number: order_number.map(|s| s.to_string()).expect("ordernumber should have a value"),
                part_number: part_number.map(|s| s.to_string()).expect("part_number should have a value"),
                due_quantity: due_quantity.expect("due_quantity should have a value"),
                assn_number: assn_number.map(|s| s.to_string()).expect("assn_number should have a value"),
                print_type: print_type.map(|s| s.to_string()).expect("print_type should have a value"),
                notes: notes.map(|s| s.to_string()),
            });
        }
    }

    Ok(orders)
}


#[tauri::command]
async fn print(order: Order, print_order_row: PrintOrderRow, user: String, serial_number: String) -> Result<String, &'static str> {
    let vc_exe_path = "C:\\Program Files (x86)\\Visual CUT 11\\Visual CUT.exe";
    let word_exe_path = "C:\\Program Files\\Microsoft Office\\root\\Office16\\WINWORD.EXE";
    let pdf_exe_path = "C:\\CustomPrograms\\labelSerialNumberProject\\install\\PDFtoPrinter.exe";
    let printer_name = "PXS-PRN-SHOP-BRTHR";

    let file_serial_number_result =  get_serial_number().await;
    let file_serial_number;
    match file_serial_number_result {
        Ok(v) =>  file_serial_number = v,
        Err(e) => return Err(e),
    }

    if print_order_row.print_type == "BOM" {
        let report_path = "\\\\pxsvsapp01\\eciShared\\Shop Order Processing\\BOMRPTv2.rpt";
        let status: ExitStatus;
        if order.part_number == order.assn_number {
            status = Command::new(vc_exe_path)
                .arg(format!("-e {}", report_path))
                .arg(format!("Parm1:{}", order.assn_number))
                .arg(format!("Printer_Only:{}", printer_name))
                .arg(format!("Print_Copies:{}", order.due_quantity))
                .status()
                .expect("failed to execute process");
        } else {
            status = Command::new(vc_exe_path)
                .arg(format!("-e {}", report_path))
                .arg(format!("Parm1:{}:::{}", order.part_number, order.assn_number))
                .arg(format!("Printer_Only:{}", printer_name))
                .arg(format!("Print_Copies:{}", order.due_quantity))
                .status()
                .expect("failed to execute process");
        }
        println!("Process exited with status: {}", status);
    } else if print_order_row.print_type == "Config" {

        let search_path = "X:\\Projects\\Configuration Sheets";
        // search for config path
        let paths = finder(search_path, order.part_number);

        match paths {
            Ok(v) =>  
                for path in v {
                    let status = Command::new(word_exe_path)
                        .arg(format!("-e {} /q /n", path.display()))
                        .arg("/mFilePrintDefault /mFileCloseOrExit /mFileExit ")
                        .status()
                        .expect("failed to execute process");

                    println!("Process exited with status: {}", status);
                    break;
                },
            Err(e) => println!("error finding file: {e:?}")
        }

    } else if print_order_row.print_type == "SNL" {
        let report_path; 
        if order.part_number == order.assn_number {
            report_path = "\\\\pxsvsapp01\\eciShared\\Shop Order Processing\\SerialNumberList_v3.rpt";
        } else {
            report_path = "\\\\pxsvsapp01\\eciShared\\Shop Order Processing\\SerialNumberList_v2.rpt";
        }
        let status = Command::new(vc_exe_path)
            .arg(format!("-e {}", report_path))
            .arg(format!("Parm1:{}", order.order_number))
            .arg(format!("Printer_Only:{}", printer_name))
            .arg(format!("Print_Copies:{}", order.due_quantity))
            .status()
            .expect("failed to execute process");
        println!("Process exited with status: {}", status);
        
    } else if print_order_row.print_type.starts_with("94A") || print_order_row.print_type.starts_with("K94A") {
        let parts = print_order_row.notes.split("?");
        let collection: Vec<&str> = parts.collect();
        let report_name = collection.get(0).unwrap().to_string();
        let mut parm1 = collection.get(1);
        let parm2 = collection.get(2);
        let parm3 = collection.get(3);
        
        if report_name == "01A000038-A01" || report_name == "01A000039-A01" || report_name == "01A000052-A01" {
            if parm1 == None {
                parm1 = Some(&"$");
            }
        }

        let paths = finder("\\\\pxsvsfs01\\Production\\Manufacturing Instructions\\Crystal Label Reports", report_name.clone());

        if report_name == "01A000199-A01" { // 0.75 labels

        } else { // normal labels
            let printer_name;
            if print_order_row.print_type == "94A000003-A01" {
                printer_name = "\\\\PXSVSFS01\\2x25ZEBRA";
            } else if print_order_row.print_type == "94A000004-A01" {
                printer_name = "\\\\PXSVSFS01\\075x025_Zebra";
            } else if print_order_row.print_type == "94A000005-A01" {
                printer_name = "\\\\PXSVSFS01\\2x3ZEBRA";
            } else if print_order_row.print_type == "94A000006-A01" {
                printer_name = "\\\\PXSVSFS01\\125x25Zebra";
            } else if print_order_row.print_type == "94A000047-A01" {
                printer_name = "\\\\PXSVSFS01\\"; // TODO
            } else {
                let error = "Could not match label to a printer";
                return Err(error);
            }
            // get extension
            match paths {
                Ok(v) =>
                    for path in v {
                        let mut finalArgs: String = format!("Parm1:{} Parm2:{}", order.order_number, serial_number);
                        if let Some(a) = parm1 {
                            finalArgs = format!("{} Parm3:{}", finalArgs, a);
                        }
                        if let Some(a) = parm2 {
                            finalArgs = format!("{} Parm4:{}", finalArgs, a);
                        }
                        if let Some(a) = parm3 {
                            finalArgs = format!("{} Parm5:{}", finalArgs, a);
                        }
                        let status = Command::new(vc_exe_path)
                                .arg("-e")
                                .arg(format!("{}", path.display()))
                                .arg(format!("{}", finalArgs))
                                .arg(format!("Printer_Only:{}", printer_name))
                                .status()
                                .expect("failed to execute process");
                        println!("Process exited with status: {}", status);
                        break;
                    },
                Err(e) => println!("error finding file: {e:?}")
            }
        }

    } else if print_order_row.print_type == "Initial DOCS" {
        //parse notes
        let parts = print_order_row.notes.split("?");
        let collection: Vec<&str> = parts.collect();
        let mut search_path = collection.get(0).unwrap().to_string();
        let report_name = collection.get(1).unwrap().to_string();
        let printer_desc = collection.get(2);

        if search_path.starts_with("P:\\") {
            search_path = format!("\\\\pxsvsfs01\\Production{}", &search_path[2..]);
        }
        // serach for document
        let paths = finder(&search_path.as_str(), report_name);

        // get extension
        match paths {
            Ok(v) =>
                for path in v {
                    let extension = path.extension().unwrap();
                    let status: ExitStatus;
                    if extension == "pdf" { // deal with printer
                        status = Command::new(pdf_exe_path)
                            .arg("/s")
                            .arg(format!("{}", path.display()))
                            // .arg(format!("{}", printer_desc))
                            .status()
                            .expect("failed to execute process");
                    } else if extension == "docx" {  // deal with printer
                        status = Command::new(word_exe_path)
                            .arg(format!("-e {} /q /n", path.display()))
                            .arg("/mFilePrintDefault /mFileCloseOrExit /mFileExit")
                            .status()
                            .expect("failed to execute process");
                    } else if extension == "xlsx" { 
                        status = Command::new("powershell -command")
                            .arg(format!("start-process -filepath '{}' -verb print", path.display()))
                            .status()
                            .expect("failed to execute process");
                        
                    } else if extension == "jpg" || extension == "png" {
                        status = Command::new("powershell -command")
                            .arg(format!("mspaint /p '{}'", path.display()))
                            .status()
                            .expect("failed to execute process");
                        
                    } else {
                        status = Command::new("")
                            .status()
                            .expect("failed to execute process");
                    }

                    println!("Process exited with status: {}", status);
                    break;
                },

            Err(e) => println!("error finding file: {e:?}")
        }
    } else if print_order_row.print_type == "Final DOCS" {
        //parse notes
        let parts = print_order_row.notes.split("?");
        let collection: Vec<&str> = parts.collect();
        let mut search_path = collection.get(0).unwrap().to_string();
        let report_name = collection.get(1).unwrap().to_string();
        let parm1 = collection.get(2);
        let parm2 = collection.get(3);
        let parm3 = collection.get(4);
        let parm4 = collection.get(5);

        if search_path.starts_with("P:\\") {
            search_path = format!("\\\\pxsvsfs01\\Production{}", &search_path[2..]);
        }
        // serach for document
        let paths = finder(&search_path.as_str(), report_name);


        // get extension
        match paths {
            Ok(v) =>
                for path in v {
                    let mut finalArgs: String = format!("Parm1:{} Parm2:{}", order.order_number, serial_number);
                    if let Some(a) = parm1 {
                        finalArgs = format!("{} Parm3:{}", finalArgs, a);
                    }
                    if let Some(a) = parm2 {
                        finalArgs = format!("{} Parm4:{}", finalArgs, a);
                    }
                    if let Some(a) = parm3 {
                        finalArgs = format!("{} Parm5:{}", finalArgs, a);
                    }
                    if let Some(a) = parm4 {
                        finalArgs = format!("{} Parm6:{}", finalArgs, a);
                    }
                    let status = Command::new(vc_exe_path)
                            .arg("-e")
                            .arg(format!("{}", path.display()))
                            .arg(format!("{}", finalArgs))
                            .arg("Printer_Only:PXS-PRN-SHOP-BRTHR")
                            .status()
                            .expect("failed to execute process");
                    println!("Process exited with status: {}", status);
                    break;
                },
            Err(e) => println!("error finding file: {e:?}")
        }
    } else {
        // let output = format!("print success {}", order_number);
        let output = "Print did not match any printing option";
        // Ok(())
        return Err(output);
    }
    
    //TODO:
    //deal with blog?
    //deal with sntracker
    // 075
    // 
    
    
    
    
    
    
    
    
    
    let snur = serial_number_up(serial_number, file_serial_number);
    match snur {
        Ok(a) => Ok("good".to_string()),
        Err(e) => Err("did not count sn up"),
    }
}

// async fn serial_number_tracker() -> Result<(), &'static str>{
    // output to serialNumbreTracker.txt which contians the orders and the name of the person printing.
// }

fn serial_number_up(serial_number: String, file_serial_number: String) -> Result<(), std::io::Error> {
    let fsnn = file_serial_number.parse::<i32>().unwrap();
    let snn = serial_number.parse::<i32>().unwrap();
    if snn > fsnn {
        let mut file = File::create("../documents/SerialNumberCount.txt")?;
        file.write_all(serial_number.as_bytes())?;
    } 
    Ok(())
}

#[tauri::command]
async fn get_serial_number() -> Result<String, &'static str> {
    let mut serial_number_file = File::open("../documents/SerialNumberCount.txt")
        .map_err(|_| "Failed to open serial number file")?;

    let mut file_serial_number = String::new();
    
    serial_number_file
        .read_to_string(&mut file_serial_number)
        .map_err(|_| "Failed to read serial number file")?;

    Ok(file_serial_number)
}

fn finder(root_dir: &str, search_term: String) -> Result<Vec<PathBuf>, String> {
    let mut i = 0;  

    let mut files = Vec::new();

    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let file_name = entry.file_name().to_string_lossy();

        if file_name.contains(search_term.as_str()) {
            println!("{}", entry.path().display());
            files.push(entry.path().to_path_buf());
        }
    }

    Ok(files)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_order_number_info,
            get_print_items,
            print,
            get_serial_number,

            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
