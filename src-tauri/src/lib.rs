mod settings;
mod sql;
mod structs;
mod serial_number_files;
mod print;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            sql::get_order_number_info,
            sql::get_print_items,
            sql::get_orders,
            print::print,
            print::check_printer_regex,
            serial_number_files::get_serial_number,
            serial_number_files::reset_serial_check,
            settings::save_settings,
            settings::load_settings,
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}