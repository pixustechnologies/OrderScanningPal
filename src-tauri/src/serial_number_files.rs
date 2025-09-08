use std::path::PathBuf;
use std::io::{prelude::*, Write};
use std::env;
use std::fs::{self, OpenOptions, File};
use tauri::{AppHandle, Manager, path::BaseDirectory};
use chrono::{prelude::*, Datelike, Utc};

pub fn serial_number_tracker(part_number: String, assn_number: String, serial_number: String, user: String, app_handle: &AppHandle) -> Result<(), String>{
    // check if file exists and create it 
    let doc_path;
    let file_path;
    // if the DOC_PATH is build,  we navigate AppData, else go to the .env location
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

pub async fn serial_number_up(serial_number: String,  app_handle: &AppHandle) -> Result<(), String> {
    let doc_path;
    let file_path;
    // if the DOC_PATH is build,  we navigate AppData, else go to the .env location
    if env!("DOC_PATH") == "build" {
        file_path = app_handle
            .path()
            .resolve("SerialNumberCount.txt", BaseDirectory::AppData)
            .map_err(|e| format!("Failed to resolve SerialNumberCount path: {}", e))?;
    } else {
        doc_path = env!("DOC_PATH");
        file_path = PathBuf::from(doc_path).join("SerialNumberCount.txt");
    }
    let file_serial_number;
    match internal_get_serial_number(&app_handle).await {
        Ok(v) =>  file_serial_number = v,
        Err(e) => return Err(format!("Error reading file serial number: {}", e)),
    }
    let fsnn = file_serial_number.parse::<i32>().unwrap();
    let snn = serial_number.parse::<i32>().unwrap();
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
pub async fn get_serial_number(app_handle: AppHandle) -> Result<String, String> {
    return internal_get_serial_number(&app_handle).await;
}

async fn internal_get_serial_number(app_handle: &AppHandle) -> Result<String, String> {
    let doc_path;
    let file_path;
    // if the DOC_PATH is build,  we navigate AppData, else go to the .env location
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

const LAST_RESET_WEEK_FILE: &str = "last_reset_week.txt";

#[tauri::command]
pub async fn reset_serial_check(app_handle: AppHandle) -> Result<(), String> {
    return handle_serial(&app_handle).await;
}

async fn handle_serial(app_handle: &AppHandle)  -> Result<(), String> {
    let serial;
    match internal_get_serial_number(&app_handle).await {
        Ok(v) =>  serial = v,
        Err(e) => return Err(e),
    }

    let now = Utc::now();
    let current_week = format!("{}-{:02}", &now.iso_week().year(), &now.iso_week().week()); // 2025-36

    let last_reset_week = get_weekly_reset_date(app_handle)
        .map_err(|e| format!("Failed to get weekly reset date: {}", e))?;


    if current_week != last_reset_week {
        // reset serial_number
        let doc_path = env!("DOC_PATH");
        let file_path;
        // if the DOC_PATH is build,  we navigate AppData, else go to the .env location
        if doc_path == "build" {
            file_path = app_handle
                .path()
                .resolve("SerialNumberCount.txt", BaseDirectory::AppData)
                .map_err(|e| format!("Failed to resolve SerialNumberCount path: {}", e))?;
        } else {
            file_path = PathBuf::from(doc_path).join("SerialNumberCount.txt");
        }
        // CHECK IF WEEKLY OR IF YEARLY
        let mut sn = serial.parse().unwrap_or(0);
        let sn_string;
        if current_week.chars().nth(3) == last_reset_week.chars().nth(3) { // weekly
            sn = (sn/10000)+1;
            sn_string = format!("{}0101", sn);
        } else { // yearly
            sn = (sn/1000000)+1;
            sn_string = format!("{}010101", sn);
        }
        let width = serial.len();
        let new_serial = format!("{:0>width$}", sn_string, width = width);

        let mut file = File::create(&file_path).map_err(|e| format!("Failed to create SerialNumberCount: {}", e))?;  
        file.write_all(new_serial.as_bytes()).map_err(|e| format!("Failed to write to SerialNumberCount: {}", e))?;

        // now reset week value
        let file_path_time;
        // if the DOC_PATH is build,  we navigate AppData, else go to the .env location
        if doc_path == "build" {
            file_path_time = app_handle
                .path()
                .resolve(LAST_RESET_WEEK_FILE, BaseDirectory::AppData)
                .map_err(|e| format!("Failed to resolve LAST_RESET_WEEK_FILE path: {}", e))?;
        } else {
            file_path_time = PathBuf::from(doc_path).join(LAST_RESET_WEEK_FILE);
        }

        let mut file_time = File::create(&file_path_time).map_err(|e| format!("Failed to create LAST_RESET_WEEK_FILE: {}", e))?;  
        file_time.write_all(current_week.as_bytes()).map_err(|e| format!("Failed to write to LAST_RESET_WEEK_FILE: {}", e))?;
    }

    Ok(())
}

fn get_weekly_reset_date(app_handle: &AppHandle) -> Result<String, String> {
    let doc_path = env!("DOC_PATH");
    let file_path_week;
    // if the DOC_PATH is build,  we navigate AppData, else go to the .env location
    if env!("DOC_PATH") == "build" {
        file_path_week = app_handle
            .path()
            .resolve(LAST_RESET_WEEK_FILE, BaseDirectory::AppData)
            .map_err(|e| format!("Failed to resolve last_reset_week path: {}", e))?;
    } else {
        file_path_week = PathBuf::from(doc_path).join(LAST_RESET_WEEK_FILE);
    }
    if !fs::exists(&file_path_week).expect("Can't check existence of last_reset_week") {
        match create_last_reset_week(&file_path_week) {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
        }
    }
    
    let mut serial_number_file = File::open(file_path_week)
        .map_err(|_| "Failed to open last_reset_week file")?;

    let mut last_reset_week = String::new();
    
    serial_number_file
        .read_to_string(&mut last_reset_week)
        .map_err(|_| "Failed to read last_reset_week file")?;

    Ok(last_reset_week)
}

fn create_last_reset_week(path: &PathBuf) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    let start = "2021-01";
    file.write_all(start.as_bytes())?;
    Ok(())
}