use std::process;

//sudo netstat -tulnp
//sudo iptables -t mangle -A OUTPUT -p udp --sport 59477 -j MARK --set-mark 1234
//sudo ip route add default dev shroud-tun table 200
//sudo ip rule add not fwmark 1234 table 200
//sudo ip rule add table main suppress_prefixlength 0

//wireguard подход через метку и таблицы
pub fn setup_network(port: u16, tun_name: &str) {
    println!("PORT: {port}");

    if let Some(err) = ensure_rt_table_exists("200 vpn_table").err() {
        panic!("Failed to setup network: {err}");
    }

    if let Some(err) = add_fmark(port).err() {
        panic!("Failed to setup network: {err}");
    }

    if let Some(err) = add_default_route_to_table(tun_name, 200).err() {
        panic!("Failed to setup network: {err}");
    }

    if let Some(err) = add_rule_for_not_marked(200).err() {
        panic!("Failed to setup network: {err}");
    }

    if let Some(err) = add_rule_for_main().err() {
        panic!("Failed to setup network: {err}");
    }
}

pub fn cleanup_vpn_rules(port: u16) {
    if let Some(err) = del_main_table_rule().err() {
        panic!("Failed to clean network: {err}");
    }
    
    if let Some(err) = del_rule_for_not_marked().err() {
        panic!("Failed to clean network: {err}");
    }
    
    if let Some(err) = del_route_from_vpn_table().err() {
        panic!("Failed to clean network: {err}");
    }
    
    if let Some(err) = del_marker(port).err() {
        panic!("Failed to clean network: {err}");
    }
}

fn ensure_rt_table_exists(line: &str) -> Result<(), std::io::Error> {
    let output = process::Command::new("grep")
        .arg("-F")
        .arg(line)
        .arg("/etc/iproute2/rt_tables")
        .output()?;

    if output.status.success() {
        println!("Table exists: {}", String::from_utf8_lossy(&output.stdout));
        return Ok(());
    }

    process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo '{}' | sudo tee -a /etc/iproute2/rt_tables > /dev/null",
            line
        ))
        .status()?;
    println!("Table {} added", line);
    Ok(())
}

//sudo iptables -t mangle -A OUTPUT -p udp --sport 59477 -j MARK --set-mark 1234
fn add_fmark(port: u16) -> Result<(), String> {
    let cmd = format!(
        "iptables -t mangle -A OUTPUT -p udp --sport {} -j MARK --set-mark 1234",
        port
    );

    println!("{}", cmd);

    run_cmd(cmd.as_str())
}

//sudo ip route add default dev shroud-tun table 200
fn add_default_route_to_table(tun_name: &str, table_id: u16) -> Result<(), String> {
    let cmd = format!("ip route add default dev {} table {}", tun_name, table_id);

    println!("{}", cmd);

    run_cmd(cmd.as_str())
}

//sudo ip rule add not fwmark 1234 table 200
fn add_rule_for_not_marked(table_id: u16) -> Result<(), String> {
    let cmd = format!("ip rule add not fwmark 1234 table {}", table_id);

    println!("{}", cmd);

    run_cmd(cmd.as_str())
}

//sudo ip rule add table main suppress_prefixlength 0
fn add_rule_for_main() -> Result<(), String> {
    let cmd = "ip rule add table main suppress_prefixlength 0";

    println!("{}", cmd);

    run_cmd(cmd)
}

fn del_main_table_rule() -> Result<(), String> {
    let cmd = "sudo ip rule del suppress_prefixlength 0 table main";
    println!("{}", cmd);

    run_cmd(cmd)
}

fn del_rule_for_not_marked() -> Result<(), String> {
    let cmd = "ip rule del not fwmark 1234 table 200";

    println!("{}", cmd);

    run_cmd(cmd)
}

fn del_route_from_vpn_table() -> Result<(), String> {
    let cmd = "ip route del default dev shroud-tun table 200";
    println!("{}", cmd);

    run_cmd(cmd)
}

fn del_marker(port: u16) -> Result<(), String> {
    let cmd = format!(
        "iptables -t mangle -D OUTPUT -p udp --sport {} -j MARK --set-mark 1234",
        port
    );
    println!("{}", cmd);
    
    run_cmd(cmd.as_str())
}

// pub fn cleanup_vpn_rules(port: u16) {
//     // Удаляем правило с suppress_prefixlength
//     let _ = process::Command::new("sudo")
//         .args([
//             "ip",
//             "rule",
//             "del",
//             "table",
//             "main",
//             "suppress_prefixlength",
//             "0",
//         ])
//         .output();
// 
//     // Удаляем правило с fwmark 1234
//     let _ = process::Command::new("sudo")
//         .args(["ip", "rule", "del", "not", "fwmark", "1234", "table", "200"])
//         .output();
// 
//     // Удаляем маршрут из таблицы 200
//     let _ = process::Command::new("sudo")
//         .args([
//             "ip",
//             "route",
//             "del",
//             "default",
//             "dev",
//             "shroud-tun",
//             "table",
//             "200",
//         ])
//         .output();
// 
//     // Удаляем маркировку трафика в iptables
//     let _ = process::Command::new("sudo")
//         .args([
//             "iptables",
//             "-t",
//             "mangle",
//             "-D",
//             "OUTPUT",
//             "-p",
//             "udp",
//             "--sport",
//             port.to_string().as_str(), // Здесь нужно использовать тот же порт, что и при создании
//             "-j",
//             "MARK",
//             "--set-mark",
//             "1234",
//         ])
//         .output();
// }

/////////////////////////////////////////////////////////////////////////////////////////////////////
// async fn cleanup_routing(server_ip: String) {
//     let route_output = Command::new("ip")
//         .arg("route")
//         .arg("del")
//         .arg(server_ip)
//         .output()
//         .await
//         .expect("Failed to execute IP ROUTE command");
//
//     if !route_output.status.success() {
//         eprintln!(
//             "Failed to set route: {}",
//             String::from_utf8_lossy(&route_output.stderr)
//         );
//     }
// }

//простой подход, default маршрут поверх всего
// async fn setup_routing(server_ip: &str) -> error::Result<()> {
//     let interface = get_default_interface().await;
//     let gateway = get_gateway().await;
//
//     let route_output = Command::new("ip")
//         .arg("route")
//         .arg("add")
//         .arg("0.0.0.0/0")
//         .arg("via")
//         .arg("10.0.0.1")
//         .arg("dev")
//         .arg(TUN_NAME)
//         .output()
//         .await
//         .expect("Failed to execute IP ROUTE command");
//
//     if !route_output.status.success() {
//         eprintln!(
//             "Failed to set route: {}",
//             String::from_utf8_lossy(&route_output.stderr)
//         );
//     }
//
//     let cidr = format!("{}/32", server_ip);
//     Command::new("ip")
//         .args([
//             "route",
//             "add",
//             &cidr,
//             "via",
//             gateway.as_str(),
//             "dev",
//             interface.as_str(),
//         ])
//         .status()
//         .await
//         .expect("Failed to add route to server");
//
//     println!(
//         "Route to server added: {} via {} dev {}",
//         server_ip, gateway, interface
//     );
//
//     Ok(())
// }
//
// //ip route show default | awk '{print $5}'
// async fn get_default_interface() -> String {
//     let output = Command::new("ip")
//         .args(["route", "show", "default"])
//         .output()
//         .await
//         .expect("Failed to execute 'ip route' command");
//
//     if output.status.success() {
//         let output_str = String::from_utf8_lossy(&output.stdout);
//         let interface = output_str.split_whitespace().nth(4).unwrap_or("unknown");
//         println!("Default interface: {}", interface);
//         interface.to_string()
//     } else {
//         panic!("Error: Could not determine default interface");
//     }
// }
//
// //ip route show default | awk '{print $3}'
// async fn get_gateway() -> String {
//     let output = Command::new("ip")
//         .args(["route", "show", "default"])
//         .output()
//         .await
//         .expect("Failed to execute 'ip route' command");
//
//     if output.status.success() {
//         let output_str = String::from_utf8_lossy(&output.stdout);
//         let interface = output_str.split_whitespace().nth(2).unwrap_or("unknown");
//         println!("Gateway: {}", interface);
//         interface.to_string()
//     } else {
//         panic!("Error: Could not determine default interface");
//     }
// }

fn run_cmd(cmd: &str) -> Result<(), String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let status = process::Command::new(parts[0])
        .args(&parts[1..])
        .status()
        .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?;

    if !status.success() {
        return Err(format!(
            "Command `{}` failed with exit code {:?}",
            cmd,
            status.code()
        ));
    }

    Ok(())
}
