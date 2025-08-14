// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// use tauri::ipc::Response;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};
use futures::TryStreamExt;
use walkdir::WalkDir;
use chrono::prelude::*;
use std::process::{Command, ExitStatus};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::io::prelude::*;
use std::io::BufReader;
use std::env;
use std::fs::{self, OpenOptions, File};
use printers::{get_default_printer, get_printer_by_name, get_printers};
use dotenvy::dotenv;
use tauri::{AppHandle, Manager, Emitter, path::BaseDirectory};
use std::ptr::null_mut;


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
    notes: String,
}

#[derive(Deserialize)]
struct PrintOrderRow {
    id: i32,
    print_type: String,
    notes: String,
}

#[derive(Serialize, Deserialize)]
struct Settings {
  font_size: i32,
  dark_mode: bool,
  part_list: Vec<String>,
}


#[tauri::command]
async fn get_orders() -> Result<Vec<Order>, String> {
    let mut client = sql_setup().await?;

    let query =
    "SELECT DISTINCT om.ORDNUM_10, om.PRTNUM_10, om.DUEQTY_10, 
        CASE pm.TYPE_01 WHEN 'S'        THEN pm.PRTNUM_01
                                        ELSE om.PRTNUM_10
        END AS ASSPRT
FROM    Requirement_Detail rd, Order_Master om, Product_Structure ps, Part_Master pm
WHERE   om.ORDNUM_10 = rd.ORDNUM_11
        AND (LEFT(om.ORDNUM_10, 1) = '5' OR LEFT(om.ORDNUM_10, 1) = '7')
        AND om.DUEQTY_10 > 0 
        AND om.PLANID_10 != '000' 
        AND pm.PRTNUM_01 = ps.COMPRT_02
        AND ps.PARPRT_02 = om.PRTNUM_10
        AND om.STATUS_10 = '3'
        AND ((pm.TYPE_01 = 'S' 
                AND (LEFT(pm.PRTNUM_01, 2) = '02' OR LEFT(pm.PRTNUM_01, 3) = 'K02') )
        OR (rd.PRTNUM_11 = ps.COMPRT_02  AND (LEFT(ps.PARPRT_02, 2) = '02' OR LEFT(ps.PARPRT_02, 3) = 'K02')))

ORDER BY om.ORDNUM_10 DESC"; // om.PLANID_10 != '000' removes screws?

    let mut stream = client
        .query(query, &[])
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
async fn get_order_number_info(order_number: String) -> Result<Vec<Order>, String> {
    let mut client = sql_setup().await?;

    let query =
    "SELECT DISTINCT om.ORDNUM_10, om.PRTNUM_10, om.DUEQTY_10, 
        CASE pm.TYPE_01 WHEN 'S'        THEN pm.PRTNUM_01
                                        ELSE om.PRTNUM_10
        END AS ASSPRT
FROM    Requirement_Detail rd, Order_Master om, Product_Structure ps, Part_Master pm
WHERE   (om.ORDNUM_10 = @P1 OR om.ORDER_10 = @P1)
        AND om.ORDNUM_10 = rd.ORDNUM_11
        AND om.DUEQTY_10 > 0 
        AND pm.PRTNUM_01 = ps.COMPRT_02
        AND ps.PARPRT_02 = om.PRTNUM_10
        AND ((pm.TYPE_01 = 'S' AND (LEFT(pm.PRTNUM_01, 3) = '0' + '2' + 'A' OR LEFT(pm.PRTNUM_01, 4) = 'K' + '0' + '2' + 'A') )
            OR rd.PRTNUM_11 = ps.COMPRT_02)";
//      AND om.STATUS_10 = '3'

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
async fn get_print_items(order_number: String, app_handle: AppHandle) -> Result<Vec<PrintOrder>, String> {
    let mut client = sql_setup().await?;

    let query =
    "SELECT  om.ORDNUM_10, om.PRTNUM_10, om.DUEQTY_10, 
        CASE pm.TYPE_01 WHEN 'S'        THEN ps.COMPRT_02
                                        ELSE om.PRTNUM_10
        END AS ASSPRT,
rd.PRTNUM_11, wn.NOTES_61
FROM    Requirement_Detail rd, Windows_Notes wn, Order_Master om, Product_Structure ps, Part_Master pm
WHERE   (om.ORDNUM_10 = @P1 OR om.ORDER_10 = @P1)
        AND om.ORDNUM_10 = rd.ORDNUM_11
        AND wn.COMPRT_61 = rd.PRTNUM_11
        AND (LEFT(rd.PRTNUM_11, 3) = '94A'
            OR LEFT(rd.PRTNUM_11, 4) = 'K94A'
            OR rd.PRTNUM_11 = 'Initial DOCS'
            OR rd.PRTNUM_11 = 'Final DOCS')
        AND om.DUEQTY_10 > 0 
        AND pm.PRTNUM_01 = ps.COMPRT_02
        AND ps.PARPRT_02 = om.PRTNUM_10
        AND ( (pm.TYPE_01 = 'S' AND wn.PRTNUM_61 = ps.COMPRT_02)
        OR (rd.PRTNUM_11 = ps.COMPRT_02 AND wn.PRTNUM_61 = om.PRTNUM_10))

ORDER BY wn.MAXID";
//  AND om.STATUS_10 = '3'
       

    let mut stream = client
        .query(query, &[&order_number])
        .await
        .map_err(|e| format!("Query error: {}", e))?;

    let mut print_orders = Vec::new();
    
    // add BOM SNL and config sheet
    print_orders.push(PrintOrder { // needs ASSN
        order_number: order_number.to_string(),
        part_number: "".to_string(),
        due_quantity: 1.0, 
        assn_number: "".to_string(),
        print_type: "BOM".to_string(),
        notes: "Bill of Materials".to_string(),
    });
    print_orders.push(PrintOrder { // needs PARTNUM for locating
        order_number: order_number.to_string(),
        part_number: "".to_string(),
        due_quantity: 1.0, 
        assn_number: "".to_string(),
        print_type: "Config".to_string(),
        notes: "Configuration Sheet (if found)".to_string(),
    });
    print_orders.push(PrintOrder { // needs ORDNUM
        order_number: order_number.to_string(),
        part_number: "".to_string(),
        due_quantity: 1.0, 
        assn_number: "".to_string(),
        print_type: "SNL".to_string(),
        notes: "Serial Number List".to_string(),
    });
    
    let app_settings = load_settings(app_handle)?;

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
            let pt = print_type.map(|s| s.trim().to_string()).expect("print_type should have a value");
            if notes == Some("") || c == Some('~') || app_settings.part_list.contains(&pt) {
                continue;
            } else if c == Some('?') { 
                if let Some(last_order) = print_orders.last_mut() {
                    last_order.notes.push_str(
                        notes.expect("notes should have a value")
                    );
                }
            } else {
                print_orders.push(PrintOrder {
                    order_number: order_number.map(|s| s.to_string()).expect("ordernumber should have a value"),
                    part_number: part_number.map(|s| s.to_string()).expect("part_number should have a value"),
                    due_quantity: due_quantity.expect("due_quantity should have a value"),
                    assn_number: assn_number.map(|s| s.to_string()).expect("assn_number should have a value"),
                    print_type: print_type.map(|s| s.to_string()).expect("print_type should have a value"),
                    notes: notes.map(|s| s.to_string()).expect("notes should have a value"),
                });
            }
        }
    }

    Ok(print_orders)
}

async fn sql_setup() -> Result<Client<Compat<TcpStream>>, String> {
    // dotenv().ok(); // Load env vars

    // let host = env::var("DB_HOST").unwrap();
    // let port: u16 = env::var("DB_PORT").unwrap().parse().unwrap();
    // let user = env::var("DB_USER").unwrap();
    // let password = env::var("DB_PASSWORD").unwrap();
    // let database = env::var("DB_NAME").unwrap();

    
    let host = env!("DB_HOST");
    let port: u16 = env!("DB_PORT").parse().unwrap();
    let user = env!("DB_USER");
    let password = env!("DB_PASSWORD");
    let database = env!("DB_NAME");

    let mut config = Config::new();
    config.host(host);
    config.port(port);
    config.authentication(AuthMethod::sql_server(user, password));
    config.database(database);
    config.trust_cert();

    // Connect
    let tcp = TcpStream::connect(config.get_addr())
        .await
        .map_err(|e| format!("TCP connect error: {}", e))?;
    let tcp = tcp.compat();

    let client = Client::connect(config, tcp)
        .await
        .map_err(|e| format!("DB connect error: {}", e))?;

    Ok(client)
}

#[tauri::command]
async fn print(order: Order, print_order_row: PrintOrderRow, user: String, serial_number: String, reprint_run: bool, app_handle: AppHandle) -> Result<String, String> {
    let vc_exe_path = r"C:\Program Files (x86)\Visual CUT 11\Visual CUT.exe";
    let word_exe_path = r"C:\Program Files\Microsoft Office\root\Office16\WINWORD.EXE";
    let pdf_exe_path = r"C:\CustomPrograms\labelSerialNumberProject\install\PDFtoPrinter.exe";
    let printer_name;

    let file_serial_number;
    match internal_get_serial_number(&app_handle).await {
        Ok(v) =>  file_serial_number = v,
        Err(e) => return Err(format!("Error reading file serial number: {}", e)),
    }

    // for printer in get_printers() {
    //     println!("{:?}", printer);
    // }                  

    match get_default_printer_wmic() {
        Some(printer) => printer_name=printer,
        None => printer_name="PXS-PRN-SHOP-BRTHR".to_string(),
    }   
    println!("Default Printer: {}", printer_name); 

    if print_order_row.print_type == "BOM" {
        let report_path = r"\\pxsvsapp01\eciShared\Shop Order Processing\BOMRPTv2.rpt";
        let status: ExitStatus;
        if order.part_number == order.assn_number {
            status = Command::new(vc_exe_path)
                .arg("-e")
                .arg(report_path)
                .raw_arg(&format!("\"Parm1:{}\"", order.order_number))
                .raw_arg(&format!("\"Printer_Only:{}\"", printer_name))
                .status()
                .map_err(|e| format!("Failed to execute process: {}", e))?;
        } else {
            status = Command::new(vc_exe_path)
                .arg("-e")
                .arg(report_path)
                .raw_arg(&format!("\"Parm1:{}:::{}\"",order.part_number, order.assn_number))
                .raw_arg(&format!("\"Printer_Only:{}\"", printer_name))
                .status()
                .map_err(|e| format!("Failed to execute process: {}", e))?;
        }
        println!("Process exited with status: {}", status);
    } else if print_order_row.print_type == "Config" {

        let search_path = r"X:\Projects\Configuration Sheets";
        // search for config path
        match finder(search_path, order.part_number.clone()) {
            Ok(v) =>  
                for path in v {
                    let status = Command::new(word_exe_path)
                        .arg("-e")
                        .arg(path.display().to_string())
                        .arg("/q")
                        .arg("/n")
                        .arg("/mFilePrintDefault")
                        .arg("/mFileCloseOrExit")
                        .arg("/mFileExit")
                        .status()
                        .map_err(|e| format!("Failed to execute Word: {}", e))?;

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
            .arg("-e")
            .arg(report_path)
            .raw_arg(&format!("\"Parm1:{}\"", order.order_number))
            .raw_arg(&format!("\"Printer_Only:{}\"", printer_name))
            .raw_arg(&format!("\"Print_Copies:{}\"", order.due_quantity))
            .status()
            .map_err(|e| format!("Failed to execute process: {}", e))?;
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

        if report_name == "01A000199-A01" { // 0.75 labels TODO

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
                return Err(error.to_string());
            }
            // get extension
            match finder("\\\\pxsvsfs01\\Production\\Manufacturing Instructions\\Crystal Label Reports", report_name) {
                Ok(v) =>
                    for path in v {
                        for i in 0..(order.due_quantity as i32) {
                        let snn = serial_number.parse::<i32>().unwrap() + i;
                        let width = serial_number.len();
                        let new_serial = format!("{:0width$}", snn, width = width);

                        let mut command = Command::new(vc_exe_path);
                        command.arg("-e");
                        command.arg(path.display().to_string());

                        // Parm arguments
                        command.arg(format!("Parm1:{}", order.order_number));
                        command.arg(format!("Parm2:{}", new_serial));

                        if let Some(a) = &parm1 {
                            command.arg(format!("Parm3:{}", a));
                        }
                        if let Some(a) = &parm2 {
                            command.arg(format!("Parm4:{}", a));
                        }
                        if let Some(a) = &parm3 {
                            command.arg(format!("Parm5:{}", a));
                        }

                        command.arg(format!("Printer_Only:{}", printer_name));

                        let status = command
                            .status()
                            .map_err(|e| format!("Failed to execute process: {}", e))?;

                        if !status.success() {
                            return Err(format!("Process exited with non-zero status: {}", status));
                        }

                        println!("Printed serial: {} (exit: {})", new_serial, status);
                    }
                        break;
                    },
                Err(e) => println!("error finding file: {e:?}")
            }
            
        }

    } else if print_order_row.print_type.to_lowercase() == "initial docs" {
        //parse notes
        let parts = print_order_row.notes.split("?");
        let collection: Vec<&str> = parts.collect();
        let mut search_path = collection.get(0).unwrap().to_string();
        let report_name = collection.get(1).unwrap().to_string();
        let mut printer_desc = collection.get(2).map_or("", |v| v).to_string();
        
        if printer_desc.to_lowercase() == "clr" {
            printer_desc = "PXS-PRN-LEX-CLR".to_string();
        }

        if search_path.starts_with(r"P:\") {
            search_path = format!(r"\\pxsvsfs01\Production{}", &search_path[2..]);
        } // TODO swap all drives to correct format

        // serach for document
        match finder(&search_path.as_str(), report_name) {
            Ok(v) =>
                for path in v {
                    for _i in 0..(order.due_quantity  as i32) {
                        let extension = path.extension().unwrap();
                        let status: ExitStatus;
                        if extension == "pdf" { 
                            status = Command::new(pdf_exe_path)
                                .arg("/s")
                                .arg(format!("{}", path.display()))
                                .arg(format!("{}", printer_desc))
                                .status()
                                .map_err(|e| format!("Failed to execute process: {}", e))?;
                        } else if extension == "docx" {
                            let target_printer = printer_desc.as_str();
                            match get_printer_by_name(target_printer) {
                                Some(_) => {
                                    match set_default_printer(target_printer) {
                                        Ok(_) => println!("Successfully set '{}' as default printer.", target_printer),
                                        Err(e) => eprintln!("Failed to set default printer: {:?}", e),
                                    }
                                }
                                None => {
                                    println!("Printer '{}' not found.", target_printer);
                                }
                            }

                            status = Command::new(word_exe_path)
                                .arg("-e")
                                .arg(path.display().to_string())
                                .arg("/q")
                                .arg("/n")
                                .arg("/mFilePrintDefault")
                                .arg("/mFileCloseOrExit")
                                .arg("/mFileExit")
                                .status()
                                .map_err(|e| format!("Failed to execute process: {}", e))?;
                            
                            match set_default_printer(printer_name.as_str()) {
                                Ok(_) => println!("Successfully set '{}' as default printer.", printer_name),
                                Err(e) => eprintln!("Failed to set default printer: {:?}", e),
                            }

                        } else if extension == "xlsx" { 
                            status = Command::new("powershell")
                                .arg("-Command")
                                .arg(format!(
                                    "Start-Process -FilePath '{}' -Verb Print",
                                    path.display()
                                ))
                                .status()
                                .map_err(|e| format!("Failed to execute process: {}", e))?;
                            
                        } else if extension == "jpg" || extension == "png" {
                            status = Command::new("mspaint.exe")
                                .arg("/p")
                                .arg(path.display().to_string())
                                .status()
                                .map_err(|e| format!("Failed to execute process: {}", e))?;
                        } else {
                            status = Command::new("")
                                .status()
                                .map_err(|e| format!("Failed to execute process: {}", e))?;
                        }

                        println!("Process exited with status: {}", status);
                    }
                    break;
                },

            Err(e) => println!("error finding file: {e:?}")
        }
    } else if print_order_row.print_type.to_lowercase() == "final docs" {
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
        match finder(&search_path.as_str(), report_name.clone()) {
            Ok(v) =>
                for path in v {
                    // for a specific reports that require first and last SN, and only print once
                    if report_name == "01A000207-A01" || report_name == "01A000208-A01" || report_name == "01A000209-A01" {
                        let snn = serial_number.parse::<i32>().unwrap() + (order.due_quantity as i32);
                        let width = serial_number.len();
                        let new_serial = format!("{:0width$}", snn, width = width);
                        let final_args: String = format!("Parm1:{} Parm2:{} Parm3:{}", order.order_number, serial_number, new_serial);
                        let status = Command::new(vc_exe_path)
                                .arg("-e")
                                .raw_arg(&format!("\"{}\"", path.display()))
                                .raw_arg(&format!("\"{}\"", final_args))
                                .raw_arg(&format!("\"Printer_Only:{}\"", printer_name))
                                .status()
                                .map_err(|e| format!("Failed to execute process: {}", e))?;
                        println!("Process exited with status: {}", status);
                        break;
                    }
                    for i in 0..(order.due_quantity  as i32) {
                        let snn = serial_number.parse::<i32>().unwrap() + i;
                        let width = serial_number.len();
                        let new_serial = format!("{:0width$}", snn, width = width);
                        let mut final_args: String = format!("Parm1:{} Parm2:{}", order.order_number, new_serial);
                        if let Some(a) = parm1 {
                            final_args = format!("{} Parm3:{}", final_args, a);
                        }
                        if let Some(a) = parm2 {
                            final_args = format!("{} Parm4:{}", final_args, a);
                        }
                        if let Some(a) = parm3 {
                            final_args = format!("{} Parm5:{}", final_args, a);
                        }
                        if let Some(a) = parm4 {
                            final_args = format!("{} Parm6:{}", final_args, a);
                        }
                        let status = Command::new(vc_exe_path)
                                .arg("-e")
                                .raw_arg(&format!("\"{}\"", path.display()))
                                .raw_arg(&format!("\"{}\"", final_args))
                                .raw_arg(&format!("\"Printer_Only:{}\"", printer_name))
                                .status()
                                .map_err(|e| format!("Failed to execute process: {}", e))?;
                        println!("Process exited with status: {}", status);
                    }
                    break;
                },
            Err(e) => println!("error finding file: {e:?}")
        }
    } else {
        // let output = format!("print success {}", order_number);
        let output = "Print did not match any printing option";
        // Ok(())
        return Err(output.to_string());
    }
    
    //TODO:
    //deal with blog?
    // 075
    
    
    if (print_order_row.print_type == "Final DOCS" || print_order_row.print_type.starts_with("94A") || print_order_row.print_type.starts_with("K94A")) && !reprint_run {
        match serial_number_up(serial_number.clone(), file_serial_number.clone(), &app_handle) {
            Ok(_a) =>   match serial_number_tracker(order.part_number.clone(), order.assn_number.clone(), serial_number.clone(), user.clone(), &app_handle) {
                                Ok(_a) => Ok(format!("Success")),
                                Err(e) => Err(format!("did not write to tracker: {} ", e)),
                            },
            Err(e) => Err(format!("did not count sn up: {} ", e)),
        }
    } else {
        Ok(format!("Success"))
    }
    
}

fn set_default_printer(printer_name: &str) -> std::io::Result<()> {
    Command::new("RUNDLL32")
        .arg("PRINTUI.DLL,PrintUIEntry")
        .arg("/y")
        .arg("/n")
        .arg(printer_name)
        .status()?
        .success()
        .then(|| ())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Failed to set default printer"))
}

fn get_default_printer_wmic() -> Option<String> {
    let output = Command::new("wmic")
        .args(["printer", "get", "name,", "default"])
        .output()
        .expect("Failed to run wmic");

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let line = line.trim();
        if line.to_ascii_lowercase().starts_with("default") || line.is_empty() {
            continue; // skip header or empty lines
        }

        if line.contains("TRUE") {
            // Assume fixed columns: "TRUE    printer name..."
            let parts: Vec<&str> = line.splitn(2, "TRUE").collect();
            if parts.len() == 2 {
                let printer_name = parts[1].trim();
                return Some(printer_name.to_string());
            }
        }
    }

    None
}


fn serial_number_tracker(part_number: String, assn_number: String, serial_number: String, user: String, app_handle: &AppHandle) -> Result<(), String>{
    // check if file exists and create it 
    let doc_path;
    let file_path;
    if env!("DOC_PATH") == "build" {
        file_path = app_handle
            .path()
            .resolve("serialNumberTracker.txt", BaseDirectory::AppData)
            .map_err(|e| format!("Failed to resolve settings path: {}", e))?;


    } else {
        doc_path = env!("DOC_PATH");
        file_path = PathBuf::from(doc_path).join("serialNumberTracker.txt");
    }
    if !fs::exists(&file_path).expect("Can't check existence of serialNumberTracker") {
        match create_serial_number_tracker(&file_path) {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
         }
    }

    // output to serialNumbreTracker.txt which contians the orders and the name of the person printing.
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&file_path);

    let local: DateTime<Local> = Local::now();
    let padded_date = format!("{: <12}", local.format("%Y-%m-%d"));
    let padded_part_number = format!("{: <30}", part_number);
    let padded_assn_number = format!("{: <30}", assn_number);
    let padded_serial_number = format!("{: <16}", serial_number);

    if let Err(e) = writeln!(file.map_err(|e| format!("Failed to write to serial number tracker: {}", e))?, "{}{}{}{}{}", padded_date,padded_part_number,padded_assn_number,padded_serial_number,user) {
        eprintln!("Couldn't write to file: {}", e);
        // return Err(format!("Couldn't write to file: {}", e));
    }
    Ok(())
}

fn create_serial_number_tracker(path: &PathBuf) -> Result<(), std::io::Error>{
    let mut file = File::create(path)?;
    let header = "Date        Model Number                  Part Number                   Serial Number   Initials \n";
    file.write_all(header.as_bytes())?;
    Ok(())
}

fn serial_number_up(serial_number: String, file_serial_number: String, app_handle: &AppHandle) -> Result<(), String> {
    let doc_path;
    let file_path;
    if env!("DOC_PATH") == "build" {
        file_path = app_handle
            .path()
            .resolve("SerialNumberCount.txt", BaseDirectory::AppData)
            .map_err(|e| format!("Failed to resolve SerialNumberCount path: {}", e))?;


    } else {
        doc_path = env!("DOC_PATH");
        file_path = PathBuf::from(doc_path).join("SerialNumberCount.txt");
    }
    let fsnn = file_serial_number.parse::<i32>().unwrap();
    let snn = serial_number.parse::<i32>().unwrap() + 1;
    if snn > fsnn {
        let width = serial_number.len();
        let new_serial = format!("{:0width$}", snn, width = width);

        let mut file = File::create(&file_path).map_err(|e| format!("Failed to create SerialNumberCount: {}", e))?;
        
        file.write_all(new_serial.as_bytes()).map_err(|e| format!("Failed to write to SerialNumberCount: {}", e))?;
    } 
    Ok(())
}

fn create_serial_number_count(path: &PathBuf) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    let start = "001010129";
    file.write_all(start.as_bytes())?;
    Ok(())
}

#[tauri::command]
async fn get_serial_number(app_handle: AppHandle) -> Result<String, String> {
    return internal_get_serial_number(&app_handle).await;
}

async fn internal_get_serial_number(app_handle: &AppHandle) -> Result<String, String> {
    let doc_path;
    let file_path;
    if env!("DOC_PATH") == "build" {
        file_path = app_handle
            .path()
            .resolve("SerialNumberCount.txt", BaseDirectory::AppData)
            .map_err(|e| format!("Failed to resolve SerialNumberCount path: {}", e))?;


    } else {
        doc_path = env!("DOC_PATH");
        file_path = PathBuf::from(doc_path).join("SerialNumberCount.txt");
    }
    if !fs::exists(&file_path).expect("Can't check existence of serialNumberCount") {
        match create_serial_number_count(&file_path) {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
         }
    }
    
    let mut serial_number_file = File::open(file_path)
        .map_err(|_| "Failed to open serial number file")?;

    let mut file_serial_number = String::new();
    
    serial_number_file
        .read_to_string(&mut file_serial_number)
        .map_err(|_| "Failed to read serial number file")?;

    Ok(file_serial_number)
}

fn finder(root_dir: &str, search_term: String) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let file_name = entry.file_name().to_string_lossy();

        if file_name.contains(search_term.as_str()) {
            println!("path: {}", entry.path().display());
            files.push(entry.path().to_path_buf());
        }
    }

    Ok(files)
}

#[tauri::command]
fn save_settings(settings: Settings, app_handle: AppHandle) -> Result<(), String> {
    let doc_path;
    let file_path;
    if env!("DOC_PATH") == "build" {
        file_path = app_handle
            .path()
            .resolve("appSettings.json", BaseDirectory::AppData)
            .map_err(|e| format!("Failed to resolve settings path: {}", e))?;

        // fs::create_dir_all(&app_data_path).map_err(|e| e.to_string())?;

    } else {
        doc_path = env!("DOC_PATH");
        file_path = PathBuf::from(doc_path).join("appSettings.json");
    }
    println!("dm{} fs{}", settings.dark_mode, settings.font_size);
    let json_string = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize Settings to JSON: {}", e))?;

    let mut file = File::create(&file_path)
        .map_err(|e| format!("Failed to create file {}: {}", &file_path.display(), e))?;
    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("Failed to write to file {}: {}", &file_path.display(), e))?;

    //update frontend
    app_handle
        .emit("settings-updated", ())
        .map_err(|e| format!("Failed to emit settings-updated event: {}", e))?;

    Ok(())
}

#[tauri::command]
fn load_settings(app_handle: AppHandle) -> Result<Settings, String> {
    let doc_path;
    let file_path;
    if env!("DOC_PATH") == "build" {
        file_path = app_handle
            .path()
            .resolve("appSettings.json", BaseDirectory::AppData)
            .map_err(|e| format!("Failed to resolve settings path: {}", e))?;


    } else {
        doc_path = env!("DOC_PATH");
        file_path = PathBuf::from(doc_path).join("appSettings.json");
    }


    if !fs::exists(&file_path).expect("Can't check existence of appSettings") {
        create_app_settings(&file_path).map_err(|e| format!("Failed to create app ettings: {}", e))?;
    }

    let file = File::open(&file_path)
        .map_err(|_| "Failed to open settings file")?;
    let reader = BufReader::new(file);

    let json_value: Value = serde_json::from_reader(reader)
        .map_err(|_| "Failed to process json file")?;
    let settings: Settings = serde_json::from_value(json_value)
        .map_err(|e| format!("Failed to parse JSON into Settings: {}", e))?;

    Ok(settings)
}

fn create_app_settings(path: &PathBuf) -> Result<(), std::io::Error>{
    let settings = Settings {
        font_size: 16,
        dark_mode: false,
        part_list: Vec::new(),
    };
    
    let json_string = serde_json::to_string_pretty(&settings)?;

    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let mut file = File::create(path)?;
    file.write_all(json_string.as_bytes())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_order_number_info,
            get_print_items,
            print,
            get_serial_number,
            get_orders,
            save_settings,
            load_settings,
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}