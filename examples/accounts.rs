use std::io::{BufReader, Read, BufRead, Write, Result, Error, ErrorKind};
use std::collections::HashMap;

use cached_struct::*;

#[derive(Debug, Default)]
struct Accounts(HashMap<String, i64>);

impl Cache for Accounts {
    fn save<W: Write>(&self, mut writer: W) -> Result<()> {
        for (name, balance) in &self.0 {
            writer.write_fmt(format_args!("{}:{}\n", name, balance))?;
        }
        Ok(())
    }
    fn load<R: Read>(reader: R) -> Result<Self> {
        let r = BufReader::new(reader);
        let mut ret = HashMap::new();

        for line in r.lines() {
            let line = line?;
            let l = line.trim();

            let i = l.rfind(':').ok_or_else(|| Error::from(ErrorKind::InvalidData))?;
            
            ret.insert(l[..i].to_owned(), l[i+1..].parse().map_err(|_| Error::from(ErrorKind::InvalidData))?);
        }
        Ok(Accounts(ret))
    }
}

fn main() -> Result<()> {
    let mut cached_accounts = Cached::<Accounts>::new("test/accounts.txt")?;
    cached_accounts.do_mut(|i| i.0.insert("test".to_owned(), -40))?;

    println!("{:?}", cached_accounts.get()?.0);

    std::thread::sleep(std::time::Duration::new(4, 0));

    println!("{:?}", cached_accounts.get()?.0);

    Ok(())
}
