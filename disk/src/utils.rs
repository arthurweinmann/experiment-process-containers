use cmd::exec;

const FIND_UMOUNTED_CMD: &str =
    "lsblk | \
     grep disk | \
     grep -Eo ^[a-zA-Z0-9]+ | \
     xargs -I % sh -c 'mount | grep -o % > /dev/null 2>&1 || echo %'| \
     xargs -I % sh -c 'btrfs filesystem show | grep -o % > /dev/null 2>&1 || echo %'";

pub fn list_umounted() -> Result<Option<Vec<String>>, exec::CommandError> {
    let output = exec::BashCommand::new_sh(FIND_UMOUNTED_CMD).run_utf8()?;
    let iter = output.trim_matches('\n').trim().split(" ");
    let mut retour: Option<Vec<String>> = None;

    for s in iter {
        if !s.is_empty() {
            if let Some(ref mut v) = retour {
                // ref mut so v is not moved but is a reference to the vec inside Option
                v.push(String::from(s));
            } else {
                retour = Some(vec![String::from(s)]);
            }
        }
    }

    Ok(retour)
}

#[cfg(test)]
mod tests {
    // cd disk && export RUST_BACKTRACE=full; cargo test -- --nocapture --test-threads=1 list_umounted
    use super::list_umounted;

    #[test]
    fn test_list_umounted() {
        let res = list_umounted().unwrap();
        println!("{:?}", res);
    }
}
