use core::fmt;
use std::{error::Error, fmt::format, path::PathBuf};
use clap::{builder::Str, Parser, Subcommand};

use rppal::spi::Spi;

pub struct LEDRing<'a>{
    num_leds:u8,
    brightness:f32,
    current_rgb_values: Vec<u8>,
    bus: &'a mut Spi,
}

#[derive(Parser, Debug)]
#[command(name = "LEDRingController", 
          version="0.4", 
          about = "Controlls an RGB LED Ring", 
          long_about = None)]
pub struct LEDRingCliArgs{
    ///Number of leds
    #[arg(short, long)]
    pub leds: u8,

    ///Brightness of the LEDs ranging from 0-1; optional
    #[arg(short, long)]
    pub brightness: Option<f32>,

    ///Optional path to a config.json file
    #[arg(short, long, value_name = "Path")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    /// Subcommand to control LEDs
    pub command: LEDRingCommands
}

#[derive(Subcommand, Debug)]
pub enum LEDRingCommands{
    #[command(about = "Setting color for single LED at index to RGB values.")]
    SetSingleRGB{
        /// LED index, 0 based
        index:u8,
        /// red value, 0-255; optional
        #[arg(short, long)]
        r: u8,
        /// green value, 0-255; optional
        #[arg(short, long)]
        g: u8,
        /// blue value, 0-255; optional
        #[arg(short, long)]
        b: u8,
    },

    #[command(about = "Setting color for single LED at index to hex value.")]
    SetSingleHEX{
        /// LED index, 0 based
        index:u8,
        /// HEX value
        #[arg(short, long)]
        hex: String
    },

    #[command(about = "Setting color for multiple LEDs for indices to RGB values.")]
    SetMultipleRGB{
        /// LED index, 0 based
        #[arg(short, long, num_args = 0..=8, value_delimiter = ' ')]
        indices:Vec<u8>,
        /// red value, 0-255; optional
        #[arg(short, long)]
        r: u8,
        /// green value, 0-255; optional
        #[arg(short, long)]
        g: u8,
        /// blue value, 0-255; optional
        #[arg(short, long)]
        b: u8
    },
    #[command(about = "Setting color for multiple LEDs for indicies to hex value.")]
    SetMultipleHEX{
        /// LED index, 0 based
        #[arg(short, long, num_args = 0..=8, value_delimiter = ' ')]
        indices:Vec<u8>,
        /// HEX value
        #[arg(short, long)]
        hex: String
    },
}

#[derive(Debug)]
pub enum LEDRingError{
    IoError(rppal::spi::Error),
    ParseError(String),
    ValueError(String),
}

impl fmt::Display for LEDRingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            LEDRingError::IoError(e) => write!(f, "I/O error: {}", e),
            LEDRingError::ParseError(e) => write!(f, "Error while parsind: {}", e),
            LEDRingError::ValueError(e) => write!(f, "Value error: {}", e)
        }
    }
}

impl From<LEDRingError> for String{
    fn from(value: LEDRingError) -> Self {
        match value{
            LEDRingError::IoError(e) => Self::from("I/O error: {e}"),
            LEDRingError::ParseError(e) => Self::from("Error while parsind: {e}"),
            LEDRingError::ValueError(e) => Self::from("Value error: {e}")
        }
    }
}

impl Error for LEDRingError {}

impl<'a> LEDRing<'a> {
    
    pub fn new(bus: &'a mut Spi, num_led:u8, brightness:Option<f32>) -> Self{
        if let Some(value) = brightness{
            return LEDRing{ num_leds: num_led, 
                            brightness: value, 
                            current_rgb_values: vec![0; 3*num_led as usize], 
                            bus};
   
        }else{
            return LEDRing{ num_leds: num_led, 
                            brightness: 1.0, 
                            current_rgb_values: vec![0; 3*num_led as usize], 
                            bus};
        }
    }

    pub fn show(&mut self) -> Result<(), LEDRingError>{
        self.apply_brightness();
        println!("Writing to LED Ring");
        let buffer = self.get_spi_data();
        match self.bus.write(&buffer){
            Ok(written) => {
                println!("{:?}", written);
                Ok(())
            },
            Err(e) => Err(LEDRingError::IoError(e))
        }
    }

    pub fn set_brigthness(&mut self, brightness:f32) -> Result<(), LEDRingError>{
        if(brightness > 0.0 && brightness < 1.0){
            self.brightness = brightness;
            Ok(())
        }else{
            Err(LEDRingError::ValueError(String::from("Brightness can not be smaller than 0 or greater than 1, brightness: {brightness}")))
        }   
    }

    pub fn brightness(&self) -> f32{
        self.brightness
    }

    pub fn set_led_values_rgb(&mut self, led_idx:u8, values: &Vec<u8>) -> Result<(), LEDRingError>{
        if(values.len() != 3){
            return Err(LEDRingError::ParseError(String::from("Expected 3 values, but received {values.len()}")))
        }

        for i in 0..3{
            self.current_rgb_values[led_idx as usize * 3 + i as usize] = values[i as usize];
        }

        Ok(())
    }

    pub fn get_led_values_rgb(&mut self, idx: u8) -> Option<Vec<u8>>{
        if(idx > self.num_leds){
            None
        }else{
            let mut result:Vec<u8> = Vec::with_capacity(3);
            for i in 0..3  {
                // self.current_rgb_values[idx * 3 .. idx * 3 + 3]
                result[i] = self.current_rgb_values[idx as usize * 3 + i as usize];
            }
            Some(result)
        }
    }

    pub fn set_led_values_hex(&mut self, led_idx:u8, hex_value: &u32) -> Result<(), LEDRingError>{

        let rgb = convert_hex_to_rgb(hex_value)?;

        self.set_led_values_rgb(led_idx, &rgb)
    }

    fn apply_brightness(&mut self){
        self.current_rgb_values.iter_mut().for_each(|value| {
            *value = (*value as f32 * self.brightness) as u8;
        });        
    }

    fn get_spi_data(&mut self) -> Vec<u8>{
        let mut tx = vec![0; self.current_rgb_values.len()*8];
        for ibit in 0..8  {
            let mut i = 0;
            for value in &self.current_rgb_values{
                let current_bit = (value >> ibit) & 1;
                //0x78 represents 1111000 and 0x80 is 10000000 
                //when the bit is high we get 5 bits high and 3 low, 
                //if the bit is low we get 1 high and 7 low
                let encoded_bit = current_bit * 0x78 + 0x80;
                tx[7-ibit + i*8] = encoded_bit;
                i+=1;
            }
        }
        return tx
    }

}

fn convert_hex_to_rgb(hex_value: &u32) -> Result<Vec<u8>, LEDRingError>{
    if(*hex_value >= 0 && *hex_value <= 0xFFFFFF){
        let red:u8 = ((hex_value >> 16) & 0xFF).try_into().unwrap();
        let green:u8 = ((hex_value >> 8) & 0xFF).try_into().unwrap();
        let blue:u8 = (hex_value & 0xFF).try_into().unwrap();
        Ok(vec![red, green, blue])
    }else{
        Err(LEDRingError::ParseError(String::from("HEX color value must be a positive 24bit number.")))
    }
}

fn convert_rgb_to_grb(rgb:Vec<u8>) -> Result<Vec<u8>, LEDRingError>{
    if(rgb.len() != 3){
        return Err(LEDRingError::ParseError(format!("RGB vector must have len 3 but has len {}", rgb.len())))
    }
    let mut grb = vec!(0; 3);
    grb[0] = rgb[1];
    grb[1] = rgb[0];
    grb[2] = rgb[2];
    Ok(grb)
}

pub fn convert_string_to_hex(hex_string:String) -> Result<u32, LEDRingError>{
    if(!hex_string.starts_with("#")){
        return Err(LEDRingError::ParseError(String::from("HEX values must start with a #")))
    }else{
        let hex_slice = &hex_string[1..];
        match u32::from_str_radix(hex_slice, 16){
            Ok(value) => Ok(value),
            Err(_) => Err(LEDRingError::ParseError(String::from("Error while parsing the HEX value.")))
        }
    }
}

#[cfg(test)]
mod tests{
    use rppal::spi::Spi;

    use crate::{convert_rgb_to_grb, convert_hex_to_rgb, LEDRing};
    #[test]
    fn test_hex_conversion() -> Result<(), String>{
        let hex = 0x112233;
        let rgb = convert_hex_to_rgb(&hex)?;

        if(rgb[0] == 17 && rgb[1] == 34 && rgb[2] == 51){
            Ok(())
        }else {
            Err(format!("RGB values incorrect: R:{}, G:{}, B:{}", rgb[0], rgb[1], rgb[2]))
        }
    }

    #[test]
    fn test_hex_to_grb() -> Result<(), String>{
        let hex = 0x112233;
        let rgb = convert_hex_to_rgb(&hex)?;
        if(rgb[0] != 17 && rgb[1] != 34 && rgb[2] != 51){
            return Err(format!("RGB values incorrect: R:{}, G:{}, B:{}", rgb[0], rgb[1], rgb[2]));
        }
        let grb = convert_rgb_to_grb(rgb)?;
        assert_eq!(grb.len(), 3 as usize);

        if(grb[1] == 17 && grb[0] == 34 && grb[2] == 51){
            Ok(())
        }else {
            Err(format!("GRB values incorrect: G:{}, R:{}, B:{}", grb[0], grb[1], grb[2]))
        }
    }

    // #[test]
    // fn test_set_led_values() -> Result<(), String>{
    //     let mut bus = Spi::new(rppal::spi::Bus::Spi0, rppal::spi::SlaveSelect::Ss0, 6400000, rppal::spi::Mode::Mode0).unwrap();
    //     let mut ring = LEDRing::new(&mut bus, 8, 1);
    //     let act = vec![34, 17, 51];
    //     ring.set_led_values_rgb(0, &act)?;
    //     assert_eq!(&act, &ring.current_rgb_values[0..3]);

    //     ring.set_led_values_hex(1, 0x112233)?;
    //     assert_eq!(&act, &ring.current_rgb_values[3..6]);
    //     Ok(())
    // }

}