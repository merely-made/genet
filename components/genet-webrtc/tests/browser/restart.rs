use super::*;

fn exercise<E: ScriptEngine>(server: Option<std::net::SocketAddr>, relay: bool) {
    let bind = server
        .map(|server| {
            let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
            socket.connect(server).unwrap();
            socket.local_addr().unwrap().ip()
        })
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap());
    let mut rt = wired_with::<E>(Settings {
        bind,
        gathering_timeout: Duration::from_secs(3),
        allocation_refresh_interval_cap: Some(Duration::from_secs(1)),
        ..Settings::default()
    });
    let servers: Vec<_> = server
        .into_iter()
        .map(|server| {
            serde_json::json!({
                "urls":[format!("{}:{server}",if relay {"turn"} else {"stun"})],
                "username":"rtc-test","credential":"rtc-test-password"
            })
        })
        .collect();
    let config = serde_json::json!({"iceTransportPolicy":if relay {"relay"} else {"all"},"iceServers":servers});
    run(
        &mut rt,
        &format!("var config={config},done=false,problem='';"),
    );
    run(&mut rt, include_str!("restart.js"));
    until(&mut rt, "done || problem!==''");
    assert_eq!(read(&mut rt, "problem"), "");
    assert_eq!(rt.pending_webrtc(), 0);
}

pub(super) fn ordinary<E: ScriptEngine>() {
    exercise::<E>(None, false);
}

pub(super) fn coturn<E: ScriptEngine>() {
    let server = std::env::var("MURM_RTC_TEST_SERVER")
        .expect("controlled coturn endpoint")
        .parse()
        .unwrap();
    exercise::<E>(Some(server), false);
    exercise::<E>(Some(server), true);
}
