use anchor_lang::prelude::ProgramError;

pub fn calculate_fees(amount:u64)->Result<u64,ProgramError>{
    let fee=(amount as f64 * 0.5 as f64 /100.0) as u64;
    Ok(fee)
}

pub fn calculate_tokens_out(sol_amount:u64,virtual_sol_reserves:u64,virtual_token_reserves:u64)->Result<u64,ProgramError>{
    let tokens_out=virtual_token_reserves-(virtual_sol_reserves*virtual_token_reserves)/(virtual_sol_reserves+sol_amount);
    
    Ok(tokens_out)
}

pub fn calculate_sol_out(tokens_amount:u64,virtual_sol_reserves:u64,virtual_token_reserves:u64)->Result<u64,ProgramError>{
    let sol_out=virtual_sol_reserves-(virtual_sol_reserves*virtual_token_reserves)/(virtual_token_reserves+tokens_amount);
    Ok(sol_out)
}