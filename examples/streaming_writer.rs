//! Utilizes the push-based `XmlPrinter` API directly (without building a DOM)
//! to construct XML in a stream-like manner.

use tinyxml2::XmlPrinter;

fn main() {
    // 1. Initialize a new pretty-printing XmlPrinter
    let mut printer = XmlPrinter::new();

    // 2. Stream XML header
    printer.push_header("1.0", Some("UTF-8"), Some(true));

    // 3. Stream a root element and comments
    printer.push_comment("System performance monitoring statistics");
    printer.open_element("metrics-report");
    printer.push_attribute("timestamp", "2026-07-01T04:00:00Z");

    // 4. Stream nested elements
    printer.open_element("host-info");
    printer.open_element("name");
    printer.push_text("worker-node-03");
    printer.close_element(); // </name>

    printer.open_element("os");
    printer.push_text("Linux Ubuntu 24.04");
    printer.close_element(); // </os>
    printer.close_element(); // </host-info>

    // 5. Stream system metrics (using compact attribute lists)
    printer.open_element("cpu");
    printer.push_attribute("cores", "16");
    printer.push_attribute("usage-pct", "42.5");
    printer.close_element(); // </cpu> (self-closing since it has no children/text)

    printer.open_element("memory");
    printer.push_attribute("total-gb", "64");
    printer.push_attribute("free-gb", "18.2");
    printer.close_element(); // </memory>

    // 6. Stream log section containing CDATA
    printer.open_element("recent-syslog");
    printer.push_comment("Raw log block containing unescaped characters");
    printer.push_cdata(
        "Jul  1 04:00:01 worker-node-03 systemd[1]: Started Periodic Command Scheduler.",
    );
    printer.close_element(); // </recent-syslog>

    printer.close_element(); // </metrics-report>

    // 7. Print the resulting output to stdout
    println!("--- Streamed XML Output ---");
    let xml_output = printer.as_str();
    println!("{xml_output}");
}
