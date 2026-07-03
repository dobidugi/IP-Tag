use std::time::Duration;

/// 공인 IP를 조회한다. 1차 ipify, 실패하면 2차 icanhazip로 폴백한다.
pub async fn fetch_public_ip() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .ok()?;

    // 1차: api.ipify.org (JSON)
    if let Ok(resp) = client.get("https://api.ipify.org?format=json").send().await {
        if let Ok(v) = resp.json::<serde_json::Value>().await {
            if let Some(ip) = v.get("ip").and_then(|x| x.as_str()) {
                if is_plausible_ip(ip) {
                    return Some(ip.to_string());
                }
            }
        }
    }

    // 2차: icanhazip.com (plain text)
    if let Ok(resp) = client.get("https://icanhazip.com").send().await {
        if let Ok(text) = resp.text().await {
            let ip = text.trim();
            if is_plausible_ip(ip) {
                return Some(ip.to_string());
            }
        }
    }

    None
}

/// 최소한의 형식 검증 (IPv4/IPv6 문자 구성). 엄밀한 파싱보다 오탐 방지 목적.
fn is_plausible_ip(s: &str) -> bool {
    !s.is_empty() && s.parse::<std::net::IpAddr>().is_ok()
}

/// 현재 접속 중인 WiFi의 SSID(이름)를 반환한다.
///
/// macOS 14+에서는 `networksetup -getairportnetwork`가 접속 중에도
/// "You are not associated..."를 반환하는 경우가 많다. 반면
/// `ipconfig getsummary <iface>`는 위치 권한 없이도 SSID를 노출한다.
#[cfg(target_os = "macos")]
pub fn current_ssid() -> Option<String> {
    use std::process::Command;

    let iface = wifi_interface().unwrap_or_else(|| "en0".to_string());
    let out = Command::new("/usr/sbin/ipconfig")
        .args(["getsummary", &iface])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&out.stdout);
    // 출력 예: "  SSID : 5G_LGWiFi_A52D" (키가 정확히 SSID인 줄만. BSSID 제외)
    text.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim() == "SSID" {
            let v = value.trim();
            (!v.is_empty()).then(|| v.to_string())
        } else {
            None
        }
    })
}

/// Wi-Fi 하드웨어 포트에 매핑된 디바이스명(예: en0)을 찾는다.
#[cfg(target_os = "macos")]
fn wifi_interface() -> Option<String> {
    use std::process::Command;

    let out = Command::new("/usr/sbin/networksetup")
        .arg("-listallhardwareports")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);

    // 블록 예:
    //   Hardware Port: Wi-Fi
    //   Device: en0
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.contains("Hardware Port") && line.contains("Wi-Fi") {
            for next in lines.by_ref() {
                if let Some(dev) = next.strip_prefix("Device: ") {
                    return Some(dev.trim().to_string());
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
pub fn current_ssid() -> Option<String> {
    None
}
