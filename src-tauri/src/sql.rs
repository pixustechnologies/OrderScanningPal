use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};
use futures::TryStreamExt;
use std::env;
use tauri::{AppHandle};
use crate::settings;
use crate::structs::{Order, PrintOrder};

#[tauri::command]
pub async fn get_orders() -> Result<Vec<Order>, String> {
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
pub async fn get_order_number_info(order_number: String) -> Result<Vec<Order>, String> {
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
        // so you can enter old shop orders to test

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
pub async fn get_print_items(order_number: String, app_handle: AppHandle) -> Result<Vec<PrintOrder>, String> {
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
    // so you can enter old shop orders to test
       

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
    
    let app_settings = settings::internal_load_settings(&app_handle)?;

    while let Some(item) = stream.try_next().await.map_err(|e| format!("Row error: {}", e))? {
        if let Some(row) = item.into_row() {
            let order_number: Option<&str> = row.get(0);
            let part_number: Option<&str> = row.get(1);
            let due_quantity: Option<f64> = row.get(2);
            let assn_number: Option<&str> = row.get(3);
            let print_type: Option<&str> = row.get(4);
            let notes: Option<&str> = row.get(5);
            // remove if note empty OR if starting with ~ OR if the label is in the omit list
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

pub async fn common_parts(order_number: String) -> Result<Vec<String>, String> {
    let mut client = sql_setup().await?;

    let query =
    "SELECT  om.ORDNUM_10, om.PRTNUM_10, om.DUEQTY_10, 
        CASE pm2.TYPE_01 WHEN 'S'        THEN ps.COMPRT_02
                                        ELSE om.PRTNUM_10
        END AS ASSPRT,
        CASE WHEN pm3.PMDES1_01  LIKE '%Standard Parts%'OR pm3 .PMDES1_01 LIKE '%Common Parts%' 	THEN pm3.PRTNUM_01
        											ELSE pm2.PRTNUM_01
     	END AS COMMONPART
FROM   Order_Master om, Product_Structure ps, Part_Master pm, Part_Master pm2, Product_Structure ps2, Part_Master pm3
WHERE   (om.ORDNUM_10 = @P1 OR om.ORDER_10 = @P1)
        AND om.PRTNUM_10 = pm.PRTNUM_01 
        AND om.DUEQTY_10 > 0 
        AND pm.PRTNUM_01 = ps.PARPRT_02
        AND ps.COMPRT_02 = pm2.PRTNUM_01
        AND ps2.PARPRT_02 = ps.COMPRT_02
        AND ps2.COMPRT_02 = pm3.PRTNUM_01
        AND (pm2.PMDES1_01 LIKE '%Standard Parts%' OR pm2.PMDES1_01 LIKE '%Common Parts%' OR pm3.PMDES1_01 LIKE '%Standard Parts%' OR pm3.PMDES1_01 LIKE '%Common Parts%')

ORDER BY om.ORDNUM_10 DESC";

    let mut stream = client
        .query(query, &[&order_number])
        .await
        .map_err(|e| format!("Query error: {}", e))?;

    let mut common_parts = Vec::new();

    while let Some(item) = stream.try_next().await.map_err(|e| format!("Row error: {}", e))? {
        if let Some(row) = item.into_row() {
            let common_part: Option<&str> = row.get(4);
            common_parts.push(common_part.map(|s| s.to_string()).expect("common_part should have a value"));
        }
    }

    Ok(common_parts)
}

async fn sql_setup() -> Result<Client<Compat<TcpStream>>, String> {
    //setup functcion for all SQL queries
    // getting information from the .env file
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