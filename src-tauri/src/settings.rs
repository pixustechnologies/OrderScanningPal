use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::path::PathBuf;
use std::io::prelude::*;
use std::io::BufReader;
use std::env;
use std::fs::{self, File};
use tauri::{AppHandle, Manager, Emitter, path::BaseDirectory};


#[derive(Serialize, Deserialize)]
pub struct Settings {
  font_size: i32,
  dark_mode: bool,
  pub common_parts: bool,
  pub part_list: Vec<String>,
  pub clr_printer: String,
  pub bom_path: String,
  pub snl_path: String,
  pub config_path: String,
  pub label_path: String,
  pub pdf_to_printer_path: String,
  pub label_printer_125_025: String,
  pub label_printer_2_025: String,
  pub label_printer_075_025: String,
  pub label_printer_2_3: String,
  pub label_printer_4_6: String,
}

#[tauri::command]
pub fn save_settings(settings: Settings, app_handle: AppHandle) -> Result<(), String> {
    let doc_path;
    let file_path;
    // if the DOC_PATH is build,  we navigate AppData, else go to the .env location
    if env!("DOC_PATH") == "build" {
        file_path = app_handle
            .path()
            .resolve("appSettings.json", BaseDirectory::AppData)
            .map_err(|e| format!("Failed to resolve settings path: {}", e))?;
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
pub fn load_settings(app_handle: AppHandle) -> Result<Settings, String> {
    return internal_load_settings(&app_handle);
}

pub fn internal_load_settings(app_handle: &AppHandle) -> Result<Settings, String> {
    let doc_path;
    let file_path;
    // if the DOC_PATH is build,  we navigate AppData, else go to the .env location
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
        common_parts: true,
        part_list: Vec::new(),
        clr_printer: "\\\\PXSVSFS01\\PIXUS-Lexmark CS310 Series PS3".to_string(),
        bom_path: "\\\\pxsvsapp01\\eciShared\\Shop Order Processing\\BOMRPTv2.rpt".to_string(),
        snl_path: "\\\\pxsvsapp01\\eciShared\\Shop Order Processing\\SerialNumberList_v4.rpt".to_string(),
        config_path: "X:\\Projects\\Configuration Sheets".to_string(), // this is a word path so no need
        label_path: "\\\\pxsvsfs01\\Production\\Manufacturing Instructions\\Crystal Label Reports".to_string(),
        pdf_to_printer_path: "C:\\Program Files (x86)\\PdftoPrinter\\PDFtoPrinter.exe".to_string(), 
        label_printer_125_025: "\\\\PXSVSFS01\\125x25Zebra".to_string(),
        label_printer_2_025: "\\\\PXSVSFS01\\2x25ZEBRA".to_string(),
        label_printer_075_025: "\\\\PXSVSFS01\\075x025_Zebra".to_string(),
        label_printer_2_3: "\\\\PXSVSFS01\\2x3ZEBRA".to_string(),
        label_printer_4_6: "\\\\PXSVSFS01\\ZDesigner ZD621-203dpi ZPL".to_string(),
    }; // anything to do with visual cut needs to be \\pxsvsfs01, otherwise there will be issues
    
    let json_string = serde_json::to_string_pretty(&settings)?;
    // if getting os 5 error, likely permission error for path location
    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let mut file = File::create(path)?;
    file.write_all(json_string.as_bytes())?;

    Ok(())
}
