
fn parse_args (args: vec): (String, String){
    // take args from command line and parse for -i and -o
    // if -i is present, read from file
    // if -o is present, write to file
    // if -i is not present, read from stdin
    // if -o is not present, write to stdout
    let mut input = String::new();
    let mut output = String::new();
    let mut input_file = false;
    let mut output_file = false;
    for i in 0..args.len() {
        if args[i] == "-i" {
            input_file = true;
            input = args[i + 1].clone();
        }
        if args[i] == "-o" {
            output_file = true;
            output = args[i + 1].clone();
        }
    }
    if input_file {
        println!("Input file: {}", input);
    }
    if output_file {
        println!("Output file: {}", output);
    }
    if !input_file {
        println!("Input file: stdin");
    }
    if !output_file {
        println!("Output file: stdout");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let instream, outstream = parse_args(args);

    // read in JSON either from stdin or file


}
