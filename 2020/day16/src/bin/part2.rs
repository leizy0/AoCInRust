use clap::Parser;
use day16::CLIArgs;

fn main() {
    let args = CLIArgs::parse();
    let (field_rules, own_ticket, other_tickets) = day16::read_ticket_info(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read ticket information from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();

    let valid_tickets = other_tickets
        .into_iter()
        .filter(|t| {
            t.into_iter()
                .all(|f| field_rules.iter().any(|r| r.contains(*f)))
        })
        .collect::<Vec<_>>();
    match day16::map_field_with_tickets(&field_rules, &valid_tickets) {
        Ok(map) => {
            println!("After mapped, my own ticket is:");
            for r_name in field_rules.iter().map(|r| r.name()) {
                println!(
                    "{}: {}",
                    r_name,
                    own_ticket.into_iter().nth(map[r_name]).unwrap()
                );
            }
            println!();

            let prod = field_rules
                .iter()
                .filter(|r| r.name().starts_with("departure"))
                .map(|r| own_ticket.into_iter().nth(map[r.name()]).unwrap())
                .product::<usize>();
            println!(
                "The product of field starts with \"departure\" is {}.",
                prod
            );
        }
        Err(e) => eprintln!(
            "Failed to map fields to numbers on given tickets, get error({}).",
            e
        ),
    }
}
