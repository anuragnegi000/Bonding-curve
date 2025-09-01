pub fn calculate_fees(amount:u64,fee_percentage:u64)->Result<()>{
    let fee=(amount as f64 * fee_percentage as f64 /100.0) as u64;
    return Ok(fee);
}

pub fn calculate_tokens_out(sol_amount:u64,virtual_sol_reserves:u64,virtual_token_reserves:u64)->Result<()>{
    let tokens_out=virtual_token_reserves-(virtual_sol_reserves*virtual_token_reserves)/(virtual_sol_reserves+sol_amount);
    return tokens_out;
    Ok(())
}

pub fn calculate_sol_out(tokens_amount:u64,virtual_sol_reserves:u64,virtual_token_reserves:u64)->Result<()>{
    Ok(())
}