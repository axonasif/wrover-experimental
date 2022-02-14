#[warn(unused_imports)]
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use embedded_svc::http::{self, client::*, status, Headers, Status};
use embedded_svc::io::Bytes;
use embedded_svc::wifi::*;
use embedded_svc::ping::Ping;

use esp_idf_svc::http::client::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;
use embedded_svc::ipv4;

use std::error::Error;
use std::sync::Arc;
use std::{thread, time::*};

use anyhow::bail;
use anyhow::Result;
use log::*;

//use html_parser::Dom;

//use select::document::Document;
//use select::predicate::{Attr, Class, Name, Predicate};

const SSID: &str = "test";//env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = "qwerqwer";//env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

fn main() ->  Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    //wifi part
    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new()?);
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    let mut wifi = wifi(
        netif_stack.clone(),
        sys_loop_stack.clone(),
        default_nvs.clone(),
    )?;

    let url = String::from("https://idos.idnes.cz/brno/odjezdy/vysledky/?f=Technologick%C3%BD%20park&fc=302003");

    let mut client = EspHttpClient::new_default()?;

    let response = client.get(&url)?.submit()?;
    let body: Result<Vec<u8>, _> = Bytes::<_, 64>::new(response.reader()).collect();
    for _ in 0..1000 {
        info!("About to fetch content from {}", url);

        /*
        let mut client = EspHttpClient::new_default()?;

        let response = client.get(&url)?.submit()?;

        let body: Result<Vec<u8>, _> = Bytes::<_, 64>::new(response.reader()).collect();

        let body = body?;
*/
        //let body = body?;
        info!(
            "Body:\n{:?}",
            String::from_utf8_lossy(&body).into_owned()
        );
        //let response = client.get(&url)?.submit()?;
        body = response.reader().into_iter().collect()/* .into_iter().collect()*/;


        //thread::sleep(Duration::from_millis(1000));
        // println!(
        //     "Body (truncated to 3K):\n{:?}",
        //     String::from_utf8_lossy(&body).into_owned()
        // );
        }
    println!("Hello, world!");

    drop(wifi);
    info!("Wifi stopped");

    Ok(())
}

#[allow(dead_code)]
fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Wifi configuration set, about to get status");

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}
