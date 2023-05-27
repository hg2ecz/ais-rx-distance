/* Zsolt Krüpl - 2023.05. */

use chrono::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};
mod aismod;

#[derive(Copy, Clone)]
struct ShipData {
    distance: f64,
    mmsi: u32,
    lat: f64,
    lon: f64,
    ts: u64,
    rxnum: u32,
    ais_is_ship: bool,
}

struct RecvPos {
    lat: f64,
    lon: f64,
    day: u64,
    file: File,
    mmsi_distance: HashMap<u32, ShipData>,
}

impl RecvPos {
    fn new(lat: f64, lon: f64) -> Self {
        RecvPos {
            lat,
            lon,
            day: 0,
            file: File::create("/dev/null").unwrap(),
            mmsi_distance: HashMap::new(),
        }
    }

    // distance = acos(sin(lat1)*sin(lat2)+cos(lat1)*cos(lat2)*cos(lon2-lon1))*6371; -- fokban!
    fn distance(&mut self, ais: aismod::Ais, ts: u64, aisrow: &str) {
        if ts / 86400 != self.day {
            self.day = ts / 86400;

            let mut all_distances: Vec<ShipData> = self.mmsi_distance.values().cloned().collect();
            all_distances.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());

            for dd in all_distances.iter().take(10) {
                let out = format!(
                    "# SUMMARY  distance: {:7.1} ts: {} mmsi: {:9} latlon: {:.6} {:.6} rxnum: {:4} ship: {}\n",
                    dd.distance, dd.ts, dd.mmsi, dd.lat, dd.lon, dd.rxnum, dd.ais_is_ship
                );
                print!("{out}");
                self.file.write(out.as_bytes()).unwrap();
            }
            self.file.flush().unwrap();

            self.mmsi_distance.clear(); // empty HashMap
            let datetime = Utc::now().format("%Y%m%d_%H%M%S");
            self.file = File::create(format!("ais-{datetime}.txt")).unwrap(); // new file
            let out = format!(
                "# RECEIVER COORDINATES: {:.6} {:.6}  (time: UTC)\n",
                self.lat, self.lon
            );
            print!("{out}");
            self.file.write(out.as_bytes()).unwrap();
        }

        let (aismmsi, aislat, aislon, aissog, aiscog) = match ais {
            aismod::Ais::Ais1 {
                mmsi,
                lat,
                lon,
                sog_kmh,
                cog_deg,
            } => (mmsi, lat, lon, sog_kmh, cog_deg),
            aismod::Ais::Ais4 { mmsi, lat, lon } => (mmsi, lat, lon, 0.0, 0.0),
            _ => (0, 0.0, 0.0, 0.0, 0.0),
        };

        if aislat > 90. || aislon > 180. || aislat == 0.0 || aislon == 0.0 {
            return;
        }

        let lat1 = aislat * std::f64::consts::PI / 180.;
        let lon1 = aislon * std::f64::consts::PI / 180.;

        let lat2 = self.lat * std::f64::consts::PI / 180.;
        let lon2 = self.lon * std::f64::consts::PI / 180.;

        let distance = 6371.
            * (lat1.sin() * lat2.sin() + lat1.cos() * lat2.cos() * (lon2 - lon1).cos()).acos();

        if let Some(&ddata_in) = self.mmsi_distance.get(&aismmsi) {
            let mut ddata = ddata_in;
            if distance > ddata.distance {
                ddata.distance = distance;
                ddata.lat = aislat;
                ddata.lon = aislon;
                ddata.ts = ts;
            }
            ddata.rxnum += 1;
            self.mmsi_distance.insert(aismmsi, ddata);
        } else {
            let ddata = ShipData {
                distance,
                mmsi: aismmsi,
                lat: aislat,
                lon: aislon,
                ts,
                rxnum: 1,
                ais_is_ship: if let aismod::Ais::Ais1 { .. } = ais {
                    true
                } else {
                    false
                },
            };
            self.mmsi_distance.insert(aismmsi, ddata);
        }

        let datetime = Utc::now().format("%Y-%m-%d %H:%M:%S");
        let plusz: String = if let aismod::Ais::Ais1 { .. } = ais {
            format!("sog: {aissog:4.1} km/h  cog: {aiscog:5.1}°")
        } else {
            // "sog:  1.7 km/h  cog: 320.8°"
            "                           ".into()
        };
        let out = format!(
            ": {distance:7.1} km,  ts: {ts} ({datetime} UTC) mmsi: {aismmsi:9} latlon: {aislat:.6} {aislon:.6} {plusz}    {aisrow}\n",
        );
        print!("{out}");
        self.file.write(out.as_bytes()).unwrap();
    }
}

fn ais_row_decoder(recv: &mut RecvPos, aismsg: &str) {
    let aismsg = aismsg.trim_end();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    match aismod::ais_decoder(aismsg) {
        Err(etxt) => eprintln!("ERROR: {etxt} :: {aismsg}"),
        Ok(ais_dec) => match ais_dec {
            aismod::Ais::Ais1 { .. } => recv.distance(ais_dec, ts, aismsg),
            aismod::Ais::Ais4 { .. } => recv.distance(ais_dec, ts, aismsg),
            _ => (),
        },
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Kell a vevő LAT LON paramétere.");
        std::process::exit(255);
    }

    let socket = UdpSocket::bind("127.0.0.1:10110").unwrap();
    let mut recv = RecvPos::new(args[1].parse().unwrap(), args[2].parse().unwrap());

    let mut buf = [0; 500];
    loop {
        let (amt, _src) = socket.recv_from(&mut buf).unwrap();
        let lines = String::from_utf8(buf[..amt].into()).unwrap();
        for line in lines.split('\n') {
            if line.len() > 20 {
                ais_row_decoder(&mut recv, &line);
            }
        }
    }
}
