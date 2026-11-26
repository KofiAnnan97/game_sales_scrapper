use errors::input::UserInputError;
use std::io;
use std::io::Write;

// CLI-only functions (writes to the console)
pub fn get_u16_input(info_text: &str) -> Result<u16, UserInputError> {
    let mut port_str = String::new();
    print!("{} ", info_text);
    io::stdout().flush()?;
    io::stdin().read_line(&mut port_str)?;
    let port_num = port_str.trim().parse::<u16>()?;
    Ok(port_num)
}

pub fn get_string_input(info_text: &str) -> Result<String, UserInputError> {
    let mut input_str = String::new();
    print!("{} ", info_text);
    io::stdout().flush()?;
    io::stdin().read_line(&mut input_str)?;
    let final_input = input_str.trim().to_string();
    //user[0..user.len() - 1]
    Ok(final_input)
}
