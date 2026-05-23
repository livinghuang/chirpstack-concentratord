use anyhow::Result;
use libconcentratord::{gnss, region};
use libloragw_sx1302::hal;

use super::super::super::super::config::{self, Region};
use super::super::{ComType, Configuration, RadioConfig};

// Linxdot LD1002 — SX1302 + 2x SX1250 + SX1261 (LBT helper, optional)
//
// Hardware reference (from on-device /etc/lora/reset_lgw.sh on image 2.0.0.05-OPEN):
//   SX1302 reset      = /dev/gpiochip0, GPIO 15
//   SX1302 power_en   = /dev/gpiochip0, GPIO 23
//   SX1261 reset      = /dev/gpiochip0, GPIO 17 (LBT/spectral scan helper)
//   SPI device        = /dev/spidev0.0
//   I2C device        = /dev/i2c-1 (STTS751 temp sensor at 0x39; may be absent on some SKUs)
//
// TX gain LUT borrowed from semtech_sx1302css923gw1 (same SX1250, same AS923 region).
pub fn new(conf: &config::Configuration) -> Result<Configuration> {
    let region = conf.gateway.region.unwrap_or(Region::AS923);

    let tx_min_max_freqs = match region {
        Region::AS923 => region::as923::TX_MIN_MAX_FREQS.to_vec(),
        Region::AS923_2 => region::as923_2::TX_MIN_MAX_FREQS.to_vec(),
        Region::AS923_3 => region::as923_3::TX_MIN_MAX_FREQS.to_vec(),
        Region::AS923_4 => region::as923_4::TX_MIN_MAX_FREQS.to_vec(),
        Region::AU915 => region::au915::TX_MIN_MAX_FREQS.to_vec(),
        Region::CN470 => region::cn470::TX_MIN_MAX_FREQS.to_vec(),
        Region::EU868 => region::eu868::TX_MIN_MAX_FREQS.to_vec(),
        Region::IN865 => region::in865::TX_MIN_MAX_FREQS.to_vec(),
        Region::KR920 => region::kr920::TX_MIN_MAX_FREQS.to_vec(),
        Region::RU864 => region::ru864::TX_MIN_MAX_FREQS.to_vec(),
        Region::US915 => region::us915::TX_MIN_MAX_FREQS.to_vec(),
        _ => return Err(anyhow!("Region not supported: {}", region)),
    };

    let gps = conf.gateway.model_flags.contains(&"GNSS".to_string());

    Ok(Configuration {
        radio_count: 2,
        clock_source: 0,
        full_duplex: false,
        lora_multi_sf_bandwidth: 125000,
        radio_config: vec![
            // radio_0 — TX-capable
            RadioConfig {
                tx_min_max_freqs,
                radio_type: hal::RadioType::SX1250,
                single_input_mode: true,
                rssi_offset: -215.4,
                rssi_temp_compensation: hal::RssiTempCompensationConfig {
                    coeff_a: 0.0,
                    coeff_b: 0.0,
                    coeff_c: 20.41,
                    coeff_d: 2162.56,
                    coeff_e: 0.0,
                },
                tx_enable: true,
                tx_gain_table: vec![
                    hal::TxGainConfig { rf_power: 0,  pa_gain: 0, pwr_idx: 0,  ..Default::default() },
                    hal::TxGainConfig { rf_power: 12, pa_gain: 0, pwr_idx: 15, ..Default::default() },
                    hal::TxGainConfig { rf_power: 13, pa_gain: 0, pwr_idx: 16, ..Default::default() },
                    hal::TxGainConfig { rf_power: 14, pa_gain: 0, pwr_idx: 17, ..Default::default() },
                    hal::TxGainConfig { rf_power: 15, pa_gain: 0, pwr_idx: 19, ..Default::default() },
                    hal::TxGainConfig { rf_power: 16, pa_gain: 0, pwr_idx: 20, ..Default::default() },
                    hal::TxGainConfig { rf_power: 17, pa_gain: 0, pwr_idx: 22, ..Default::default() },
                    hal::TxGainConfig { rf_power: 18, pa_gain: 1, pwr_idx: 1,  ..Default::default() },
                    hal::TxGainConfig { rf_power: 19, pa_gain: 1, pwr_idx: 2,  ..Default::default() },
                    hal::TxGainConfig { rf_power: 20, pa_gain: 1, pwr_idx: 3,  ..Default::default() },
                    hal::TxGainConfig { rf_power: 21, pa_gain: 1, pwr_idx: 4,  ..Default::default() },
                    hal::TxGainConfig { rf_power: 22, pa_gain: 1, pwr_idx: 5,  ..Default::default() },
                    hal::TxGainConfig { rf_power: 23, pa_gain: 1, pwr_idx: 6,  ..Default::default() },
                    hal::TxGainConfig { rf_power: 24, pa_gain: 1, pwr_idx: 7,  ..Default::default() },
                    hal::TxGainConfig { rf_power: 26, pa_gain: 1, pwr_idx: 8,  ..Default::default() },
                    hal::TxGainConfig { rf_power: 27, pa_gain: 1, pwr_idx: 9,  ..Default::default() },
                ],
            },
            // radio_1 — RX-only
            RadioConfig {
                radio_type: hal::RadioType::SX1250,
                single_input_mode: false,
                rssi_offset: -215.4,
                rssi_temp_compensation: hal::RssiTempCompensationConfig {
                    coeff_a: 0.0,
                    coeff_b: 0.0,
                    coeff_c: 20.41,
                    coeff_d: 2162.56,
                    coeff_e: 0.0,
                },
                tx_enable: false,
                tx_min_max_freqs: vec![],
                tx_gain_table: vec![],
            },
        ],
        gnss: match gps {
            true => conf.gateway.get_gnss_dev_path(&gnss::Device::new("/dev/ttyAMA0")),
            false => gnss::Device::None,
        },
        gnss_family: gnss::Family::GenericNmea,
        com_type: ComType::Spi,
        com_path: conf.gateway.get_com_dev_path("/dev/spidev0.0"),
        i2c_path: None,
        i2c_temp_sensor_addr: None,
        sx1302_reset_pin: conf.gateway.get_sx1302_reset_pin("/dev/gpiochip0", 15),
        sx1302_power_en_pin: conf.gateway.get_sx1302_power_en_pin("/dev/gpiochip0", 23),
        sx1261_reset_pin: conf.gateway.get_sx1261_reset_pin("/dev/gpiochip0", 17),
        ..Default::default()
    })
}
