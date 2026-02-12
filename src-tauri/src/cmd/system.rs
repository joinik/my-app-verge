// use anyhow::Result;
// use network_interface::{NetworkInterface, NetworkInterfaceConfig};
// use tokio::net::TcpListener;

// /// Get system hostname
// #[tauri::command]
// pub fn get_system_hostname() -> String {
//     gethostname::gethostname().to_string_lossy().to_string()
// }

// #[tauri::command]
// pub fn get_network_interface() -> Result<serde_json::Value, String> {
//     let interfaces = NetworkInterface::show().map_err(|e| e.to_string())?;
//     let interface_json = interfaces
//         .into_iter()
//         .map(|iface| {
//             serde_json::json!({
//                 "name": iface.name,
//                 "ip_count": iface.addr.len(),
//             })
//         })
//         .collect::<Vec<_>>();
//     Ok(serde_json::json!(interface_json))
// }

// /// Check if port is in use
// #[tauri::command]
// pub fn is_port_in_use(port: u16) -> Result<bool, String> {
//     use std::net::TcpListener;
//     match TcpListener::bind(("127.0.0.1", port)) {
//         Ok(_) => Ok(false),
//         Err(_) => Ok(true),
//     }
// }

// /// Open web URL
// #[tauri::command]
// pub fn open_web_url(url: &str)->Result<(),String> {
//     if let Err(e) = tauri_plugin_shell::open(url){
//         return Err(format!("Failed to open URL: {}", e))
//     } 
//     OK(()) // 相当于返回void

// }

// #[tauri::command]
// pub fn get_network_interfaces_info()->Result<serde_json::
// Value, String >{
//     use network_interface::NetworkInterface;
//     let interfaces = NetworkInterface::show().map_err(|e| e.to_string())?
//     let interfaces_info = interfaces.into_iter().map(|iface|{
//     let ipv4s = iface.addr.iter().filter(|addr| addr.addr.is_ipv4()).map(|addr|addr.addr.to_string )
//     })
// }