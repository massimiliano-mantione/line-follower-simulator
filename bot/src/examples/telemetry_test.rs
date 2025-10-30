use crate::blocking_api::*;

pub fn telemetry_test_run() {
    let data: Vec<_> = (0..=255u8).into_iter().collect();

    console_log("writing test.bin");
    write_plain_file("test", &data);

    console_log("writing csv-test.csv");
    let spec = [
        csv::col("u1", csv::C_U8),
        csv::col("u2", csv::C_U8),
        csv::col(".", csv::PAD_8),
        csv::col("u3", csv::C_U8),
        csv::col("s1", csv::C_I16),
        csv::col("s2", csv::C_I16),
    ];
    write_csv_file("csv-test", &data, &spec);
}
