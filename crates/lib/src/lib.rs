use std::{env::args, error::Error};

pub fn get_args() -> Result<(String, Vec<String>), Box<dyn Error>> {
    let prog_name_and_args = args().collect::<Vec<_>>();

    let prog_name = prog_name_and_args
        .get(0)
        .ok_or(Into::<Box<dyn Error>>::into("Cant get the program name"))?
        .to_string();

    let args = prog_name_and_args
        .get(1..)
        .ok_or(Into::<Box<dyn Error>>::into(
            "Cant get the program arguments",
        ))?
        .to_vec();

    Ok((prog_name, args))
}
