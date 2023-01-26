use std::fs::read_to_string;
use regex::Regex;
use std::error::Error;
use std::env; 
use std::io::{BufRead,BufReader,Stdin}; 
use std::process;
use std::fs;
use manylines::OneLine;
//
mod myerror; 
mod manylines; 
//
pub struct Config{
    pattern: String,
    path:    String,
    ignore_case: bool,
    color: bool,
    afterlines: usize,
    beforelines:usize, 
}

#[derive(PartialEq)]
enum MyStatus{
    Printing,
    Silent,
}

impl Config{   
    pub fn build_iterator(mut args: impl Iterator<Item = String>,) -> Result<Config, &'static str>{
        args.next(); 
        let (pattern,path) ={
            let mut expect_value= false;  
            let mut parameters = args.filter(|stringa| spot_parameters_and_values(stringa, &mut expect_value));
            let pattern = match  parameters.next(){
                Some(arg) => arg,
                None => return Err("Missing pattern"),
            };
            let mut path = " ".to_string(); 
            loop {
                let more_path = match parameters.next(){
                    Some(arg) => format!{"  {arg}"},
                    None => break,
                };
                path.push_str(&more_path)
            }
            let path = if path != " " {path.trim().to_string()} else {path}; 
            (pattern,path)
        };
        let ignore_case = env::var("IGNORE_CASE").is_ok() || env::args().filter(|stringa| stringa=="-i").count()>0 ;
        let color = env::args().filter(|stringa| stringa=="--color").count()>0; 
        let afterlines:usize =  search_string_interator(std::env::args(), &"-A").unwrap_or(0);
        let beforelines:usize = search_string_interator(std::env::args(), &"-B").unwrap_or(0);
        Ok(Config{pattern,path, ignore_case,color,afterlines, beforelines})
    }
        
    pub fn read_from_stdin(self: &Config) ->bool{
        self.path == " "
    }
    
    pub fn how_many_files(self: &Config) -> usize {
        self.path.split_whitespace().count() 
    }
   }


fn search_string_interator<T>(mut iterator:  impl Iterator<Item=String>, tosearch: &str ) -> Option<T> 
where 
    T: std::str::FromStr,
{
    let mut res:Option<T> =  None; 
    let mut found = false;
    loop {
        let _item = match iterator.next(){
           Some(arg) => {
               if arg[..] == *tosearch{
                  found = true;
                  break 
               }
           },
           None => break, 
        };
    } 
    if found{
        let valarg = iterator.next().expect("option {tosearch} must be followed by its value");
    
        let value = match valarg[..].parse::<T>(){
            Ok(val) => val,
            Err(_)=> {
                eprintln!{"{valarg} is invalid value  for {tosearch} option"};
                process::exit(1);
            }  
            
        };
        res = Some(value);  
    } 
    res
}

pub fn run (conf: &Config, ifile: usize) -> Result<(),Box<dyn Error>>{
    let res= if conf.how_many_files() == 0 {
          let fin = std::io::stdin(); 
          let buffer:BufReader<Stdin> = std::io::BufReader::new(fin);
          grep_from_buffer(conf, buffer, "")? 
        } else {
            let files:Vec<&str> = conf.path.split_whitespace().collect(); 
            let path = files[ifile];
            let fin = fs::File::open(path)?; 
            let buffer=  std::io::BufReader::new(fin); 
            let displaypath = if conf.how_many_files() == 1 {""} else{path}; 
            grep_from_buffer(conf, buffer, displaypath)?
            //grep_from_string_of_lines(conf, ifile)  
        }; 
        Ok(res)
}
    
pub fn grep_from_buffer<T: std::io::Read>(conf: &Config, buffer: BufReader<T>, path: &str) -> Result<(),Box<dyn Error>>{
    //
    let re = match conf.ignore_case{
        true => {
            let pattern = format!("(?i){}",conf.pattern);
            Regex::new(&pattern).unwrap()
        },
        false => Regex::new(&conf.pattern).unwrap(),
    }; 
    let max_storedlines = conf.beforelines+1;
    let mut linee = buffer.lines(); 
    let mut veclinee:Vec<OneLine> =  vec![];
    let pathformat = if path=="" {""} else {":"};
    let mut countafterlines = 0;  
    let mut status =MyStatus::Silent;
    loop {
        let linea = match linee.next() {
            Some(linea) => linea,
            None => {
                loop{
                    let linea = match veclinee.pop(){
                        Some(line) => line,
                        None => break,
                    };
                    if linea.prints(){
                        let linea = linea.to_string();
                        if conf.color{
                            let temp = re.replace_all(&linea, "\x1B[31m$0\x1B[0m").to_string(); 
                            println!{"\x1B[33m{path}\x1B[0m{pathformat}{temp}"}
                        } else {
                            println!{"{}{}{}", path, pathformat, linea}
                        }
                    }
                }
                break}, 
        }; 
        if veclinee.len() >= max_storedlines {
            let linea = veclinee.pop().expect("qui non dovrebbe crashare");  
            if linea.prints(){
                status = MyStatus::Printing; 
                let linea = linea.to_string();
                if conf.color{
                    let temp = re.replace_all(&linea, "\x1B[31m$0\x1B[0m").to_string(); 
                    println!{"\x1B[33m{path}\x1B[0m{pathformat}{temp}"}
                } else {
                    println!{"{}{}{}", path, pathformat, linea}
                }
            } else {
                if status == MyStatus::Printing {
                    println!{"---"}
                }
                status = MyStatus::Silent 
            }
        } 
        let linea = linea?; 
        let is_match = re.is_match(&linea);
        countafterlines = if is_match {conf.afterlines+1} else if countafterlines >= 1 {countafterlines - 1} else {0}; 
        if is_match{
            for i in 0..veclinee.len(){
                veclinee[i].toggle_prints();
            }
        }
        let prints = countafterlines > 0;
        let newline =OneLine::new(linea, prints, is_match); 
        let mut temp = vec![newline]; 
        //dbg!(veclinee.len());
        if veclinee.len() > 0{
            let slicelen = (max_storedlines-1).min(veclinee.len()); 
            temp.extend_from_slice(&veclinee[0..slicelen]);
        } 
        veclinee = temp;
        //dbg!{&veclinee};
    };
    //dbg!(conf.beforelines);
    Ok(())
}
    
pub fn grep_from_string_of_lines(conf: &Config, ifile: usize) -> Result<(), Box<dyn Error>>{
    let files:Vec<&str> = conf.path.split_whitespace().collect(); 
    let path = files[ifile];
    let linee = read_to_string(path)?;
    let re = match conf.ignore_case{
        true => {
            let pattern = format!("(?i){}",conf.pattern);
            Regex::new(&pattern).unwrap()
        },
        false => Regex::new(&conf.pattern).unwrap(),
    };
    let matchinglines = search_re(&re, &linee);
    let matchingiterator = matchinglines.iter();
    let pathinfo = match  conf.how_many_files() > 1{
            true => path,
            false=>"",
        };
    if conf.color {
                print_linee_with_color(&re, matchingiterator, pathinfo);
    } else {
        let pathformat = if pathinfo=="" {""} else {":"};
        for l in matchingiterator{
            println!{"{pathinfo}{pathformat}{l}"};
        };    
    };
    //dbg!(conf.beforelines);
       
    Ok(())
}

    
pub fn search<'a> (pattern: &str, linee: &'a str) -> Vec<&'a str> {
    linee
        .lines()
        .filter(|linea| linea.contains(pattern))
        .collect()
}

pub fn search_case_insensitive<'a> (pattern: &str, linee: &'a str) -> Vec<&'a str>{
    let pattern = pattern.to_lowercase(); 
    linee
        .split('\n')
        .filter(|linea| linea.to_lowercase().contains(&pattern))
        .collect() 
}

pub fn search_re<'a> (re: &regex::Regex, linee: &'a str) -> Vec<&'a str>{
    linee.lines()
    .filter(|linea| re.is_match(linea))
    .collect::<Vec<&str>>()
}

 fn print_linee_with_color<'a, T: Iterator<Item= &'a&'a str>> (re: &regex::Regex, mut linee: T, path: &str){ 
    loop{
        let linea = match linee.next() {
            Some(str) => str.to_string(),
            None => break,
        };
        let temp = re.replace_all(&linea, "\x1B[31m$0\x1B[0m").to_string();
        if path ==""{
            println!{"{temp}"}
        } else {
        println!{"\x1B[33m{path}:\x1B[0m {temp}"}
        };
    };    
}

fn spot_parameters_and_values(stringa: &String, expect_value: &mut bool) -> bool {
    let res = !stringa.starts_with("-") &&  !*expect_value; 
    if *expect_value{
        *expect_value = false;
    };
    let slice = &stringa[..]; 
    *expect_value = match  slice{
        "-A"   => true, 
        "-B" => true,
        &_  => false, 
    }; 
    res
} 

#[cfg(test)]
use myerror::NotImplementedError;
//use myerror::READING_STDIN;  
#[cfg(test)]
mod tests{
    static  LINEE: &str = "/\nPoche linee di testo\nBastano poche linee\nper provare il caso\ncase sensitive\nbasta cosi";
    //
    //
    use super::*;
    #[test]
    fn one_results(){
        let pattern = "parola";
        let linee = r#"\
            questo è un testo
            di prova
            per controllare che la 
            parola prova venga
            trovata da search
            "#;
        assert_eq!{vec!["            parola prova venga"],search(pattern, linee)};
    }
    
    #[test]
    fn case_sensitive(){
        let pattern: &str = "Basta";
        assert_eq!{vec!["Bastano poche linee"],search(pattern, LINEE)}; 
    } 
    
    #[test]
    fn case_insensitive(){
        let pattern: &str = "basTa";
        assert_eq!{vec!["Bastano poche linee","basta cosi"],search_case_insensitive(pattern, LINEE)};       
    }
    
    #[test] 
    fn regex_case_sensitive(){
        let re = Regex::new(r"che\s").unwrap();
        assert_eq!{vec!["Poche linee di testo", "Bastano poche linee"], search_re(&re, LINEE) };
    }
   //
   //
   #[test]
   fn regex_case_insensitive(){
       let re = Regex::new(r"(?i)^basta").unwrap();
       assert_eq!{vec!["Bastano poche linee", "basta cosi"], search_re(&re, LINEE)};
   } 
   //
   //
   #[test]
   fn not_implemented_error(){
       static TEST_FEATURE: &str ="Fake Feature";
       let fakefn = ||{
           let errore:Result<(),Box<dyn Error>> = Err(Box::new(NotImplementedError{feature: TEST_FEATURE}));  
            errore
       };
       if let Err(e) = fakefn(){
            assert_eq!("feature Fake Feature not yet implemented",e.to_string())
       }
   }
   #[test]
   fn togle_expect_value(){
        let mut check:bool = true;
        let _dummy = spot_parameters_and_values(&"pippo".to_string(),&mut check); 
        assert_eq!(check, false)
   }
   #[test]
   fn set_expect_value(){
        let mut check:bool = false;
        let four_args = vec!["ciao".to_string(),"-A".to_string(),"5".to_string(),"pippo".to_string(),"-i".to_string(),"paperino".to_string()]; 
        let params  = four_args.iter()
            .filter(|stringa| spot_parameters_and_values(stringa, &mut check) )
            .collect::<Vec<&String>>();
        dbg!(&params);
        assert_eq!(vec![&"ciao".to_string(),&"pippo".to_string(), &"paperino".to_string()],params);
   }
   #[test]
   fn search_usize_option_parameter(){
       let four_args = vec!["ciao","-A","5","pippo","-i","paperino"]; 
       let myres:Option<usize> = search_string_interator(four_args.iter().map(|item| item.to_string()), &"-A");
       let value = myres.expect("error reading value");
       assert_eq!(value, 5);
   }
   #[test] 
   fn search_string_option_parameter(){
       let four_args = vec!["ciao","-A","prova","pippo","-i","paperino"]; 
       let myres:Option<String> = search_string_interator(four_args.iter().map(|item| item.to_string()), &"-A");
       let value = myres.expect("error reading value");
       assert_eq!(&value[..], "prova"); 
   }
   #[test] 
   fn fail_search(){
       let four_args = vec!["ciao","-A","5","pippo","-i","paperino"]; 
       let myres:Option<usize> = search_string_interator(four_args.iter().map(|item| item.to_string()), &"-B");
       let value = myres.unwrap_or(0);
       assert_eq!(value,0); 
   }
   #[test]
   fn fail_search_string(){
        let four_args = vec!["ciao","-B","prova","pippo","-i","paperino"]; 
        let myres:Option<String> = search_string_interator(four_args.iter().map(|item| item.to_string()), &"-A");
        let value = myres.unwrap_or("".to_string());
        assert_eq!(&value[..], "");  
   }
   #[test]
   fn create_and_read_oneline(){
       let myline = String::from("rust è un linguaggio cervellotico");
       let prova = super::manylines::OneLine::new(myline, true, false); 
       assert_eq!(prova.to_string(),"rust è un linguaggio cervellotico")
   }
   #[test]
   fn toggle_oneline_prints(){
       let myline = String::from("fanculo al rust");
       let mut prova = super::manylines::OneLine::new(myline, false, false); 
       let check_1 = prova.prints();
       assert_eq!(check_1, false);
       prova.toggle_prints();
       let check_2 = prova.prints();
       assert_eq!(check_2, true)
   }
}
