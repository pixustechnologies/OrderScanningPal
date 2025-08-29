use walkdir::WalkDir;
use std::process::{Command, ExitStatus};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use printers::{get_printer_by_name, get_printers};
use tauri::{AppHandle};
use once_cell::sync::OnceCell;
use crate::settings;
use crate::serial_number_files;
use crate::structs::{Order, PrintOrderRow};


static DEFAULT_PRINTER: OnceCell<String> = OnceCell::new();

#[tauri::command]
pub async fn print(order: Order, print_order_row: PrintOrderRow, user: String, serial_number: String, reprint_run: bool, app_handle: AppHandle) -> Result<String, String> {
    let vc_exe_path = r"C:\Program Files (x86)\Visual CUT 11\Visual CUT.exe";
    let word_exe_path = r"C:\Program Files\Microsoft Office\root\Office16\WINWORD.EXE";
    let printer_name;
 
    let app_settings = settings::internal_load_settings(&app_handle)?;          
    
    match get_default_printer_cached() {
        Some(printer) => printer_name=printer.to_string(),
        None => printer_name="Brother HL-2270DW series".to_string(),
    }   
    println!("Default Printer: {}", printer_name); 

    //handle each type of print
    if print_order_row.print_type == "BOM" {
        let status: ExitStatus;
        if order.part_number == order.assn_number {
            status = Command::new(vc_exe_path)
                .arg("-e")
                .arg(app_settings.bom_path)
                .raw_arg(&format!("\"Parm1:{}\"", order.part_number))
                .raw_arg(&format!("\"Printer_Only:{}\"", printer_name))
                .status()
                .map_err(|e| format!("Failed to execute process: {}", e))?;
        } else {
            status = Command::new(vc_exe_path)
                .arg("-e")
                .arg(app_settings.bom_path)
                .raw_arg(&format!("\"Parm1:{}:::{}\"",order.part_number, order.assn_number))
                .raw_arg(&format!("\"Printer_Only:{}\"", printer_name))
                .status()
                .map_err(|e| format!("Failed to execute process: {}", e))?;
        }
        println!("Process exited with status: {}", status);
    } else if print_order_row.print_type == "Config" {
        // search for config path
        match finder(&app_settings.config_path, order.part_number.clone()) {
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
        let status = Command::new(vc_exe_path)
            .arg("-e")
            .arg(app_settings.snl_path)
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
        
        // forced to default value indicator $ (setup in crystal reports)
        if report_name == "01A000038-A01" || report_name == "01A000039-A01" || report_name == "01A000052-A01" {
            if parm1 == None {
                parm1 = Some(&"$");
            }
        }

        // match to correct printer
        let printer_name;
        if print_order_row.print_type == "94A000003-A01" {
            printer_name =  &app_settings.label_printer_2_025;
        } else if print_order_row.print_type == "94A000004-A01" {
            printer_name = &app_settings.label_printer_075_025;
        } else if print_order_row.print_type == "94A000005-A01" {
            printer_name = &app_settings.label_printer_2_3;
        } else if print_order_row.print_type == "94A000006-A01" {
            printer_name = &app_settings.label_printer_125_025;
        } else if print_order_row.print_type == "94A000047-A01" {
            printer_name = &app_settings.label_printer_4_6;
        } else {
            let error = "Could not match label to a printer";
            return Err(error.to_string());
        }
        
        // get extension
        match finder(&app_settings.label_path, report_name) {
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
                        command.raw_arg(&format!("\"Parm1:{}\"", order.order_number));
                        command.raw_arg(&format!("\"Parm2:{}\"", new_serial));

                        if let Some(a) = &parm1 {
                            command.raw_arg(&format!("\"Parm3:{}\"", a));
                        }
                        if let Some(a) = &parm2 {
                            command.raw_arg(&format!("\"Parm4:{}\"", a));
                        }
                        if let Some(a) = &parm3 {
                            command.raw_arg(&format!("\"Parm5:{}\"", a));
                        }

                        command.raw_arg(&format!("\"Printer_Only:{}\"", printer_name));

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

    } else if print_order_row.print_type.to_lowercase() == "initial docs" {
        //parse notes
        let parts = print_order_row.notes.split("?");
        let collection: Vec<&str> = parts.collect();
        let mut search_path = collection.get(0).unwrap().to_string();
        let report_name = collection.get(1).unwrap().to_string();
        let mut printer_desc = collection.get(2).map_or(printer_name.clone(), |v| v.to_string());
        
        // swap to clr printer
        if printer_desc.to_lowercase() == "clr" {
            printer_desc = app_settings.clr_printer;
        }

        search_path = swap_drive(search_path);

        // serach for document
        match finder(&search_path.as_str(), report_name) {
            Ok(v) =>
                for path in v {
                    let extension = path.extension().unwrap();
                    let status: ExitStatus;
                    if extension == "pdf" { 
                        status = Command::new(&app_settings.pdf_to_printer_path)
                            .arg("/s")
                            .arg(format!("{}", path.display()))
                            .arg(format!("{}", printer_desc))
                            .status()
                            .map_err(|e| format!("Failed to execute process: {}", e))?;
                    } else if extension == "docx" || extension == "doc" {
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
                                return Err(format!("Printer '{}' not found.", target_printer));
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

                    } else if extension == "xlsx" || extension == "xls" { 
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

        search_path = swap_drive(search_path);

        // serach for document
        match finder(&search_path.as_str(), report_name.clone()) {
            Ok(v) =>
                for path in v {
                    // for a specific reports that require first and last SN, and only print once
                    if report_name == "01A000207-A01" || report_name == "01A000208-A01" || report_name == "01A000209-A01" {
                        let snn = serial_number.parse::<i32>().unwrap() + (order.due_quantity as i32);
                        let width = serial_number.len();
                        let new_serial = format!("{:0width$}", snn, width = width);
                        let status = Command::new(vc_exe_path)
                                .arg("-e")
                                .arg(path.display().to_string())
                                .raw_arg(&format!("\"Parm1:{}\"", order.order_number))
                                .raw_arg(&format!("\"Parm2:{}\"", serial_number))
                                .raw_arg(&format!("\"Parm3:{}\"", new_serial))
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

                        let mut command = Command::new(vc_exe_path);
                        command.arg("-e");
                        command.arg(path.display().to_string());

                        // Parm arguments
                        command.raw_arg(&format!("\"Parm1:{}\"", order.order_number));
                        command.raw_arg(&format!("\"Parm2:{}\"", new_serial));

                        if let Some(a) = &parm1 {
                            command.raw_arg(&format!("\"Parm3:{}\"", a));
                        }
                        if let Some(a) = &parm2 {
                            command.raw_arg(&format!("\"Parm4:{}\"", a));
                        }
                        if let Some(a) = &parm3 {
                            command.raw_arg(&format!("\"Parm5:{}\"", a));
                        }

                        command.raw_arg(&format!("\"Printer_Only:{}\"", printer_name));

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
    } else {
        let output = format!("print did not match any printing option; {}", print_order_row.print_type);
        return Err(output.to_string());
    }
    
    // if final docs / label, count up the serial number, and record in the serial number tracker
    if (print_order_row.print_type == "Final DOCS" || print_order_row.print_type.starts_with("94A") || print_order_row.print_type.starts_with("K94A")) && !reprint_run {
        let snn = serial_number.parse::<i32>().unwrap() + (order.due_quantity as i32);
        let width = serial_number.len();
        let new_serial = format!("{:0width$}", snn, width = width);
        match serial_number_files::serial_number_up(new_serial, &app_handle).await {
            Ok(_) => println!("sn up success"),
            Err(e) => return Err(format!("did not count sn up: {}", e)),
        }
        for i in 0..(order.due_quantity  as i32) {
            let snn = serial_number.parse::<i32>().unwrap() + i;
            let width = serial_number.len();
            let new_serial = format!("{:0width$}", snn, width = width);
            match serial_number_files::serial_number_tracker(order.part_number.clone(), order.assn_number.clone(), new_serial, user.clone(), &app_handle) {
                Ok(_) => println!("sn tracker success"),
                Err(e) => return Err(format!("did not write to tracker: {}", e)),
            }
        }
    } 
    Ok(format!("Success"))
    
}

fn swap_drive(drive_path: String) -> String {
    // drive swap to real name, to be changed
    if drive_path.starts_with("P:\\") {
        format!("\\\\pxsvsfs01\\Production{}", &drive_path[2..])
    } else if drive_path.starts_with("Q:\\") {
        format!("\\\\pxsvsfs01\\Quality{}", &drive_path[2..])
    } else if drive_path.starts_with("R:\\") {
        format!("\\\\pxsvsfs01\\Purchasing{}", &drive_path[2..])
    } else if drive_path.starts_with("S:\\") {
        format!("\\\\pxsvsfs01\\Sales & Marketing{}", &drive_path[2..])
    } else if drive_path.starts_with("X:\\") {
        format!("\\\\pxsvsfs01\\UserData{}", &drive_path[2..])
    } else if drive_path.starts_with("Y:\\") {
        format!("\\\\pxsvsfs01\\Engineering{}", &drive_path[2..])
    } else {
        drive_path
    }
}

fn set_default_printer(printer_name: &str) -> std::io::Result<()> {
    // sets default printer
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

fn fetch_default_printer_wmic() -> Option<String> {
    // gets current windows default printer
    println!("Loading default printer...");
    let output = Command::new("wmic")
        .args(["printer", "get", "name,", "default"])
        .output()
        .expect("Failed to run wmic");

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let line = line.trim();
        if line.to_ascii_lowercase().starts_with("default") || line.is_empty() {
            continue; // skip header
        }

        if line.contains("TRUE") {
            // skip column TRUE
            let parts: Vec<&str> = line.splitn(2, "TRUE").collect();
            if parts.len() == 2 {
                let printer_name = parts[1].trim();
                return Some(printer_name.to_string());
            }
        }
    }

    None
}

fn get_default_printer_cached() -> Option<&'static str> {
    //caches the default printer, if the user decides to print mulitple times -> saves on runtime 2nd run
    // we could consider saving default printer like CLR printer, but could run into issue when user wants to change it, they have to change in windows and our app. 
    DEFAULT_PRINTER
        .get_or_init(|| fetch_default_printer_wmic().unwrap_or_else(|| "Unknown".to_string()))
        .as_str()
        .into()
}

fn finder(root_dir: &str, search_term: String) -> Result<Vec<PathBuf>, String> {
    // finds all files in root_dir that have serach_term
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
pub async fn check_printer_regex(printer: String) -> Result<bool, String> {
    // output list of printers
    let printer_list = get_printers();

    // gets list of printers available
    let counter = printer_list.iter().any(|p| {
        println!("{:?}", p.name);
        p.name.to_ascii_lowercase() == printer.to_ascii_lowercase()
    });
    println!("");

    Ok(counter)
}