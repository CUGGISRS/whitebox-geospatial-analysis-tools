/* 
This tool is part of the WhiteboxTools geospatial analysis library.
Authors: Dr. John Lindsay
Created: June 1, 2017
Last Modified: December 3, 2017
License: MIT
*/

use std;
use std::io::BufWriter;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::io::{Error, ErrorKind};
use std::path;
use std::u16;
use std::process::Command;
use lidar::*;
use tools::*;

/// This tool can be used to print basic information about the data contained within a LAS file, used to store LiDAR
/// data. The reported information will include including data on the header, point return frequency, and classification 
/// data and information about the variable length records (VLRs) and geokeys.
/// 
/// # Input Parameters
///
/// | Flag      | Description                                                     |
/// |-----------|-----------------------------------------------------------------|
/// | -i, input | Input LAS file.                                                 |
/// | --vlr     | Flag indicates whether to print variable length records (VLRs). |
/// | --geokeys | Flag indicates whether to print the geokeys.                    |
///
/// # Example
/// ```
/// >>./whitebox_tools -r=LidarInfo --wd=/path/to/data/ -i=file.las --vlr --geokeys
/// ```

pub struct LidarInfo {
    name: String,
    description: String,
    parameters: Vec<ToolParameter>,
    example_usage: String,
}

impl LidarInfo {
    pub fn new() -> LidarInfo { // public constructor
        let name = "LidarInfo".to_string();
        
        let description = "Prints information about a LiDAR (LAS) dataset, including header, point return frequency, and classification data and information about the variable length records (VLRs) and geokeys.".to_string();
        
        let mut parameters = vec![];
        parameters.push(ToolParameter{
            name: "Input File".to_owned(), 
            flags: vec!["-i".to_owned(), "--input".to_owned()], 
            description: "Input LiDAR file.".to_owned(),
            parameter_type: ParameterType::ExistingFile(ParameterFileType::Lidar),
            default_value: None,
            optional: false
        });

        parameters.push(ToolParameter{
            name: "Output Summary Report File".to_owned(), 
            flags: vec!["-o".to_owned(), "--output".to_owned()], 
            description: "Output HTML file for regression summary report.".to_owned(),
            parameter_type: ParameterType::NewFile(ParameterFileType::Html),
            default_value: None,
            optional: true
        });

        parameters.push(ToolParameter{
            name: "Print the variable length records (VLRs)?".to_owned(), 
            flags: vec!["--vlr".to_owned()], 
            description: "Flag indicating whether or not to print the variable length records (VLRs).".to_owned(),
            parameter_type: ParameterType::Boolean,
            default_value: None,
            optional: true
        });

        parameters.push(ToolParameter{
            name: "Print the geokeys?".to_owned(), 
            flags: vec!["--geokeys".to_owned()], 
            description: "Flag indicating whether or not to print the geokeys.".to_owned(),
            parameter_type: ParameterType::Boolean,
            default_value: None,
            optional: true
        });
        
        let sep: String = path::MAIN_SEPARATOR.to_string();
        let p = format!("{}", env::current_dir().unwrap().display());
        let e = format!("{}", env::current_exe().unwrap().display());
        let mut short_exe = e.replace(&p, "").replace(".exe", "").replace(".", "").replace(&sep, "");
        if e.contains(".exe") {
            short_exe += ".exe";
        }
        let usage = format!(">>.*{0} -r={1} --wd=\"*path*to*data*\" -i=file.las --vlr --geokeys\"
.*{0} -r={1} --wd=\"*path*to*data*\" -i=file.las", short_exe, name).replace("*", &sep);
    
        LidarInfo { name: name, description: description, parameters: parameters, example_usage: usage }
    }
}

impl WhiteboxTool for LidarInfo {
    fn get_source_file(&self) -> String {
        String::from(file!())
    }
    
    fn get_tool_name(&self) -> String {
        self.name.clone()
    }

    fn get_tool_description(&self) -> String {
        self.description.clone()
    }

    fn get_tool_parameters(&self) -> String {
        let mut s = String::from("{\"parameters\": [");
        for i in 0..self.parameters.len() {
            if i < self.parameters.len() - 1 {
                s.push_str(&(self.parameters[i].to_string()));
                s.push_str(",");
            } else {
                s.push_str(&(self.parameters[i].to_string()));
            }
        }
        s.push_str("]}");
        s
    }

    fn get_example_usage(&self) -> String {
        self.example_usage.clone()
    }

    fn run<'a>(&self, args: Vec<String>, working_directory: &'a str, verbose: bool) -> Result<(), Error> {
        let mut input_file: String = "".to_string();
        let mut output_file = String::new();
        let mut show_vlrs = false;
        let mut show_geokeys = false;
        let mut keyval: bool;
        if args.len() == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Tool run with no paramters."));
        }
        for i in 0..args.len() {
            let mut arg = args[i].replace("\"", "");
            arg = arg.replace("\'", "");
            let cmd = arg.split("="); // in case an equals sign was used
            let vec = cmd.collect::<Vec<&str>>();
            keyval = false;
            if vec.len() > 1 { keyval = true; }
            if vec[0].to_lowercase() == "-i" || vec[0].to_lowercase() == "--input" {
                if keyval {
                    input_file = vec[1].to_string();
                } else {
                    input_file = args[i+1].to_string();
                }
            } else if vec[0].to_lowercase() == "-o" || vec[0].to_lowercase() == "--output" {
                if keyval {
                    output_file = vec[1].to_string();
                } else {
                    output_file = args[i + 1].to_string();
                }
            } else if vec[0].to_lowercase() == "-vlr" || vec[0].to_lowercase() == "--vlr" {
                show_vlrs = true;
            } else if vec[0].to_lowercase() == "-geokeys" || vec[0].to_lowercase() == "--geokeys" {
                show_geokeys = true;
            }
        }

        if verbose {
            println!("***************{}", "*".repeat(self.get_tool_name().len()));
            println!("* Welcome to {} *", self.get_tool_name());
            println!("***************{}", "*".repeat(self.get_tool_name().len()));
        }

        let sep = std::path::MAIN_SEPARATOR;
        // if !working_directory.ends_with(sep) {
        //     working_directory.push_str(&(sep.to_string()));
        // }

        if !input_file.contains(sep) {
            input_file = format!("{}{}", working_directory, input_file);
        }

        if output_file.len() == 0 { output_file = input_file.replace(".las", "_summary.html"); }


        let f = File::create(output_file.clone())?;
        let mut writer = BufWriter::new(f);

        let mut s = "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">
        <head>
            <meta content=\"text/html; charset=iso-8859-1\" http-equiv=\"content-type\">
            <title>LAS File Summary</title>
            <style  type=\"text/css\">
                h1 {
                    font-size: 14pt;
                    margin-left: 15px;
                    margin-right: 15px;
                    text-align: center;
                    font-family: Helvetica, Verdana, Geneva, Arial, sans-serif;
                }
                h2 {
                    font-size: 12pt;
                    margin-left: 15px;
                    margin-right: 15px;
                    text-align: left;
                    font-family: Helvetica, Verdana, Geneva, Arial, sans-serif;
                }
                p, ol, ul, li {
                    font-size: 12pt;
                    font-family: Helvetica, Verdana, Geneva, Arial, sans-serif;
                    margin-left: 15px;
                    margin-right: 15px;
                }
                caption {
                    font-family: Helvetica, Verdana, Geneva, Arial, sans-serif;
                    font-size: 12pt;
                    margin-left: 15px;
                    margin-right: 15px;
                }
                table {
                    font-size: 12pt;
                    font-family: Helvetica, Verdana, Geneva, Arial, sans-serif;
                    font-family: arial, sans-serif;
                    border-collapse: collapse;
                    align: center;
                }
                td, th {
                    text-align: left;
                    padding: 8px;
                }
                tr:nth-child(1) {
                    border-bottom: 1px solid #333333;
                    border-top: 2px solid #333333;
                }
                tr:last-child {
                    border-bottom: 2px solid #333333;
                }
                tr:nth-child(even) {
                    background-color: #dddddd;
                }
                .numberCell {
                    text-align: right;
                }
                .headerCell {
                    text-align: center;
                }
            </style>
        </head>
        <body>
            <h1>LAS File Summary</h1>
        ";
        writer.write_all(s.as_bytes())?;

        let input = match LasFile::new(&input_file, "r") {
            Ok(lf) => lf,
            Err(_) => return Err(Error::new(ErrorKind::NotFound, format!("No such file or directory ({})", input_file))),
        };

        let s1 = &format!("<h2>File Summary</h2><p>{}", input);
        writer.write_all(s1.replace("\n", "<br>").as_bytes())?;
        
        let num_points = input.header.number_of_points;
        let mut min_i = u16::MAX;
        let mut max_i = u16::MIN;
        let mut intensity: u16;
        let mut num_first: i64 = 0;
        let mut num_last: i64 = 0;
        let mut num_only: i64 = 0;
        let mut num_intermediate: i64 = 0;
        let mut ret: u8;
        let mut nrets: u8;
        let mut p: PointData;
        let mut ret_array: [i32; 5] = [0; 5];
        let mut class_array: [i32; 256] = [0; 256];
        for i in 0..input.header.number_of_points as usize {
            p = input[i]; //.get_point_info(i);
            ret = p.return_number();
            if ret > 5 {
                // Return is too high
                ret = 5;
            }
            ret_array[(ret - 1) as usize] += 1;
            nrets = p.number_of_returns();
            class_array[p.classification() as usize] += 1;
            if nrets == 1 {
                num_only += 1;
            } else if ret == 1 && nrets > 1 {
                num_first += 1;
            } else if ret == nrets {
                num_last += 1;
            } else {
                num_intermediate += 1;
            }
            intensity = p.intensity;
            if intensity > max_i { max_i = intensity; }
            if intensity < min_i { min_i = intensity; }
        }

        // println!("\n\nMin I: {}\nMax I: {}", min_i, max_i);
        let s1 = &format!("<br>Min Intensity: {}<br>Max Intensity: {}</p>", min_i, max_i);
        writer.write_all(s1.as_bytes())?;

        s = "<h2>Point Returns Analysis</h2>";
        writer.write_all(s.as_bytes())?;

        // Point Return Table
        s = "<p><table>
        <caption>Point Return Table</caption>
        <tr>
            <th class=\"headerCell\">Return Value</th>
            <th class=\"headerCell\">Number</th>
            <th class=\"headerCell\">Percentage</th>
        </tr>";
        writer.write_all(s.as_bytes())?;

        for i in 0..5 {
            if ret_array[i] > 0 {
                let s1 = &format!("<tr>
                    <td>{}</td>
                    <td class=\"numberCell\">{}</td>
                    <td class=\"numberCell\">{}</td>
                </tr>\n",
                i + 1,
                ret_array[i],
                format!("{:.1}%", ret_array[i] as f64 / num_points as f64 * 100f64 ));
                writer.write_all(s1.as_bytes())?;
            }
        }

        s = "</table></p>";
        writer.write_all(s.as_bytes())?;

        // Point Return Table
        s = "<p><table>
        <caption>Point Position Table</caption>
        <tr>
            <th class=\"headerCell\">Return Position</th>
            <th class=\"headerCell\">Number</th>
            <th class=\"headerCell\">Percentage</th>
        </tr>";
        writer.write_all(s.as_bytes())?;

        let s1 = &format!("<tr>
            <td>Only</td>
            <td class=\"numberCell\">{}</td>
            <td class=\"numberCell\">{}%</td>
        </tr>\n",
        num_only,
        format!("{:.1}", num_only as f64 / num_points as f64 * 100f64 ));
        writer.write_all(s1.as_bytes())?;

        let s1 = &format!("<tr>
            <td>First</td>
            <td class=\"numberCell\">{}</td>
            <td class=\"numberCell\">{}%</td>
        </tr>\n",
        num_first,
        format!("{:.1}", num_first as f64 / num_points as f64 * 100f64 ));
        writer.write_all(s1.as_bytes())?;

        let s1 = &format!("<tr>
            <td>Intermediate</td>
            <td class=\"numberCell\">{}</td>
            <td class=\"numberCell\">{}%</td>
        </tr>\n",
        num_intermediate,
        format!("{:.1}", num_intermediate as f64 / num_points as f64 * 100f64 ));
        writer.write_all(s1.as_bytes())?;

        let s1 = &format!("<tr>
            <td>Last</td>
            <td class=\"numberCell\">{}</td>
            <td class=\"numberCell\">{}%</td>
        </tr>\n",
        num_last,
        format!("{:.1}", num_last as f64 / num_points as f64 * 100f64 ));
        writer.write_all(s1.as_bytes())?;

        s = "</table></p>";
        writer.write_all(s.as_bytes())?;


        // Point Classification Table
        s = "<p><table>
        <caption>Point Classification Table</caption>
        <tr>
            <th class=\"headerCell\">Classification</th>
            <th class=\"headerCell\">Number</th>
            <th class=\"headerCell\">Percentage</th>
        </tr>";
        writer.write_all(s.as_bytes())?;

        for i in 0..256 {
            if class_array[i] > 0 {
                let percent: f64 = class_array[i] as f64 / num_points as f64 * 100.0;
                let percent_str = format!("{:.*}", 1, percent);
                let class_string = convert_class_val_to_class_string(i as u8);
                let s1 = &format!("<tr>
                    <td>{}</td>
                    <td class=\"numberCell\">{}</td>
                    <td class=\"numberCell\">{}%</td>
                </tr>\n",
                class_string,
                class_array[i],
                percent_str);
                writer.write_all(s1.as_bytes())?;
            }
        }

        s = "</table></p>";
        writer.write_all(s.as_bytes())?;

        if show_vlrs {
            s = "<h2>Variable Length Records</h2>";
            writer.write_all(s.as_bytes())?;
            if input.header.number_of_vlrs > 0 {
                for i in 0..(input.header.number_of_vlrs as usize) {
                    let s1 = &format!("<p>VLR {}:<br>{}</p>", i, input.vlr_data[i].clone());
                    writer.write_all(s1.as_bytes())?;
                }
            } else {
                s = "<p>VLRs have not been set.</p>";
                writer.write_all(s.as_bytes())?;
            }
        }

        if show_geokeys {
            s = "<h2>Geokeys</h2>";
            writer.write_all(s.as_bytes())?;
            let s1 = &format!("<p>{}</p>", input.geokeys.interpret_geokeys());
            writer.write_all(s1.as_bytes())?;
        }

        s = "</body>";
        writer.write_all(s.as_bytes())?;

        let _ = writer.flush();

        if verbose {
            if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
                let output = Command::new("open")
                    .arg(output_file.clone())
                    .output()
                    .expect("failed to execute process");

                let _ = output.stdout;
            } else if cfg!(target_os = "windows") {
                // let output = Command::new("cmd /c start")
                let output = Command::new("explorer.exe")
                    .arg(output_file.clone())
                    .output()
                    .expect("failed to execute process");

                let _ = output.stdout;
            } else if cfg!(target_os = "linux") {
                let output = Command::new("xdg-open")
                    .arg(output_file.clone())
                    .output()
                    .expect("failed to execute process");

                let _ = output.stdout;
            }

            println!("Complete! Please see {} for output.", output_file);
        }

        Ok(())
    }
}
