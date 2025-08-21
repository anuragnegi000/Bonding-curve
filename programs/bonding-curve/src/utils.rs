pub fn calculate_fees(amount:u64,fee_percentage:u64)->Result<()>{
    let fee=(amount as f64 * fee_percentage as f64 /100.0) as u64;
    return Ok(fee);
}