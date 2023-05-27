/* Zsolt Krüpl - 2023. 05. */

fn extractbit(msg: &[u8], mut msg_ptr: usize, mut bitnum: usize) -> u32 {
    let mut num = 0;
    while bitnum > 0 {
        let msg_byte = msg_ptr / 6;
        let msg_bit_inv = msg_ptr % 6;
        let msg_bitct = 6 - msg_bit_inv;
        if msg_bitct <= bitnum {
            num <<= msg_bitct;
            num |= (msg[msg_byte] & (0x3f >> msg_bit_inv)) as u32;
            msg_ptr += msg_bitct;
            bitnum -= msg_bitct;
        } else {
            num <<= bitnum;
            num |= (msg[msg_byte] >> (6 - bitnum)) as u32;
            msg_ptr += bitnum;
            bitnum -= bitnum;
        }
    }
    num
}

fn extract_coord(msg: &[u8], msg_ptr: usize, bitnum: usize) -> f64 {
    let mut res_int = extractbit(msg, msg_ptr, bitnum) as i32;
    if res_int >= (1 << (bitnum - 1)) {
        res_int -= 1 << bitnum;
    }
    res_int as f64 / 600_000.
}

pub enum Ais {
    Ais1 {
        mmsi: u32,
        lon: f64,
        lat: f64,
        sog_kmh: f32,
        cog_deg: f32,
    },
    Ais4 {
        mmsi: u32,
        lon: f64,
        lat: f64,
    },
    AisUnknown,
}

// !AIVDM,1,1,,A,23Wj8j@P0vQGC5lK>ji@7wwL2HKN,0*20
pub fn ais_decoder<'a>(ais: &str) -> Result<Ais, &'a str> {
    match ais.as_bytes()[0] {
        b'!' => {
            let mut aisfields = ais.split(',');

            let _vdm_vdo = aisfields.next().ok_or("aisfield(0) missing")?;
            let pktnum_all = aisfields.next().ok_or("aisfield(1) missing")?;
            let _pktnum = aisfields.next().ok_or("aisfield(2) missing")?;
            let _pktid = aisfields.next().ok_or("aisfield(3) missing")?;
            let _channel = aisfields.next().ok_or("aisfield(4) missing")?;
            let msg_str = aisfields.next().ok_or("aisfield(5) missing")?;
            let tmp_rem_chk = aisfields.next().ok_or("aisfield(6) missing")?;
            if tmp_rem_chk.len() < 4 {
                return Err("aisfield cheksum is too short");
            }
            let chksum = &tmp_rem_chk[2..4];
            let chk_calc_num = ais[1..ais.len() - 3].bytes().fold(0, |acc, x| acc ^ x);
            if chksum != format!("{chk_calc_num:02X}") {
                return Err("chksum");
            }
            //println!("{msg_str}");
            let msgtype = msg_str.as_bytes()[0];
            if pktnum_all == "1" {
                let mut msg: Vec<u8> = msg_str.bytes().collect();
                for ch in &mut msg {
                    if (0x30..=0x57).contains(ch) || (0x60..=0x77).contains(ch) {
                        *ch -= if *ch < 0x60 { 0x30 } else { 0x38 };
                    } else {
                        return Err("six decode");
                    }
                }
                if msg.len() <= 38 / 6 {
                    return Err("ais msg len");
                };
                let mmsi = extractbit(&msg, 8, 30);
                match msgtype {
                    b'1'..=b'3' => {
                        if msg.len() != 168 / 6 {
                            return Err("ais msg len");
                        };
                        let lon = extract_coord(&msg, 61, 28);
                        let lat = extract_coord(&msg, 89, 27);
                        let sog_kmh = extractbit(&msg, 50, 10) as f32 / 10. * 1.852;
                        let cog_deg = extractbit(&msg, 116, 12) as f32 / 10.;
                        return Ok(Ais::Ais1 {
                            mmsi,
                            lon,
                            lat,
                            sog_kmh,
                            cog_deg,
                        });
                    }
                    b'4' => {
                        if msg.len() != 168 / 6 {
                            return Err("ais msg len");
                        };
                        let lon = extract_coord(&msg, 79, 28);
                        let lat = extract_coord(&msg, 107, 27);
                        return Ok(Ais::Ais4 { mmsi, lon, lat });
                    }
                    _ => return Ok(Ais::AisUnknown),
                }
            }
        }

        b'$' => (),
        _ => return Err("Unknown BS msg type"),
    }
    Ok(Ais::AisUnknown)
}
