use std::fs;
use std::path::Path;

fn main() {
    let base_dir = Path::new("tests/corpus");
    let valid_dir = base_dir.join("valid");
    let invalid_dir = base_dir.join("invalid");
    let unicode_dir = base_dir.join("unicode");

    fs::create_dir_all(&valid_dir).unwrap();
    fs::create_dir_all(&invalid_dir).unwrap();
    fs::create_dir_all(&unicode_dir).unwrap();

    // 1. Generate Valid XML Corpus (50+ files)
    let mut valid_files = Vec::new();

    // Basic structure (10 files)
    valid_files.push(("basic_01.xml", "<root></root>"));
    valid_files.push(("basic_02.xml", "<root/>"));
    valid_files.push(("basic_03.xml", "<root><child/></root>"));
    valid_files.push(("basic_04.xml", "<root><child></child></root>"));
    valid_files.push(("basic_05.xml", "<root><child1/><child2/></root>"));
    valid_files.push(("basic_06.xml", "<!-- comment --><root/>"));
    valid_files.push(("basic_07.xml", "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root/>"));
    valid_files.push(("basic_08.xml", "<root>text</root>"));
    valid_files.push(("basic_09.xml", "<root><child>nested text</child></root>"));
    valid_files.push(("basic_10.xml", "<root>mixed <child/> text</root>"));

    // Attributes (10 files)
    valid_files.push(("attr_01.xml", "<root attr=\"value\"/>"));
    valid_files.push(("attr_02.xml", "<root attr='value'/>"));
    valid_files.push(("attr_03.xml", "<root attr1=\"v1\" attr2=\"v2\"/>"));
    valid_files.push(("attr_04.xml", "<root attr=\"\"/>"));
    valid_files.push(("attr_05.xml", "<root attr=\"&quot;escaped&quot;\"/>"));
    valid_files.push(("attr_06.xml", "<root attr=\"&lt;value&gt;\"/>"));
    valid_files.push(("attr_07.xml", "<root attr=\"&amp;ampersand\"/>"));
    valid_files.push(("attr_08.xml", "<root attr=\"&apos;single-quotes&apos;\"/>"));
    valid_files.push(("attr_09.xml", "<root attr=\"space inside\"/>"));
    valid_files.push(("attr_10.xml", "<root attr=\"\nnewline\t&amp; tabs\"/>"));

    // Text & Entities (8 files)
    valid_files.push(("text_01.xml", "<root>plain text content</root>"));
    valid_files.push(("text_02.xml", "<root>escaped &lt; &gt; &amp; &quot; &apos;</root>"));
    valid_files.push(("text_03.xml", "<root>decimal entity &#65;&#66;&#67;</root>"));
    valid_files.push(("text_04.xml", "<root>hex entity &#x41;&#x42;&#x43;</root>"));
    valid_files.push(("text_05.xml", "<root>whitespace-only \n\t </root>"));
    valid_files.push(("text_06.xml", "<root>mixed text <child/> more text <child2/></root>"));
    valid_files.push(("text_07.xml", "<root>cdata and text <![CDATA[<unescaped>]]> mixed</root>"));
    valid_files.push(("text_08.xml", "<root>nested elements with text <c1><c2>deep</c2></c1></root>"));

    // CDATA Sections (6 files)
    valid_files.push(("cdata_01.xml", "<root><![CDATA[cdata content]]></root>"));
    valid_files.push(("cdata_02.xml", "<root><![CDATA[<elements> are ignored </here>]]></root>"));
    valid_files.push(("cdata_03.xml", "<root><![CDATA[&lt; entities are literal &lt;]]></root>"));
    valid_files.push(("cdata_04.xml", "<root><![CDATA[]]></root>"));
    valid_files.push(("cdata_05.xml", "<root><![CDATA[multiple]]><![CDATA[sections]]></root>"));
    valid_files.push(("cdata_06.xml", "<root><![CDATA[brackets ] ]> are fine]]></root>"));

    // Comments & Declarations (8 files)
    valid_files.push(("comment_01.xml", "<root><!-- simple comment --></root>"));
    valid_files.push(("comment_02.xml", "<root><!-- multi-line\ncomment --></root>"));
    valid_files.push(("comment_03.xml", "<root><!-- adjacent --> <!-- comments --></root>"));
    valid_files.push(("comment_04.xml", "<!-- before --><root><!-- inside --></root><!-- after -->"));
    valid_files.push(("decl_01.xml", "<?xml version=\"1.0\"?><root/>"));
    valid_files.push(("decl_02.xml", "<?xml version=\"1.0\" encoding=\"utf-8\"?><root/>"));
    valid_files.push(("decl_03.xml", "<?xml version=\"1.0\" standalone=\"yes\"?><root/>"));
    valid_files.push(("decl_04.xml", "<?custom-process instructions?><root/>"));

    // Deep / Wide (8 files)
    // Deep Nesting (10, 50, 99, 100 levels)
    let deep_10 = build_nested_xml(10);
    valid_files.push(("deep_10.xml", &deep_10));
    let deep_50 = build_nested_xml(50);
    valid_files.push(("deep_50.xml", &deep_50));
    let deep_99 = build_nested_xml(99);
    valid_files.push(("deep_99.xml", &deep_99));
    let deep_100 = build_nested_xml(100);
    valid_files.push(("deep_100.xml", &deep_100));

    // Wide Trees (100, 500, 1000 siblings)
    let wide_100 = build_wide_xml(100);
    valid_files.push(("wide_100.xml", &wide_100));
    let wide_500 = build_wide_xml(500);
    valid_files.push(("wide_500.xml", &wide_500));
    let wide_1000 = build_wide_xml(1000);
    valid_files.push(("wide_1000.xml", &wide_1000));

    // A large-ish file (100KB)
    let large_100k = build_large_xml(1000);
    valid_files.push(("large_100k.xml", &large_100k));

    for (name, content) in &valid_files {
        fs::write(valid_dir.join(name), content).unwrap();
    }

    // 2. Generate Invalid XML Corpus (30+ files)
    let mut invalid_files = Vec::new();

    // Unclosed tags (6 files)
    invalid_files.push(("unclosed_01.xml", "<root>"));
    invalid_files.push(("unclosed_02.xml", "<root><child>"));
    invalid_files.push(("unclosed_03.xml", "<root><!-- comment"));
    invalid_files.push(("unclosed_04.xml", "<root><![CDATA[cdata"));
    invalid_files.push(("unclosed_05.xml", "<?xml version=\"1.0\""));
    invalid_files.push(("unclosed_06.xml", "<root attr=\"unclosed value"));

    // Mismatched tags (5 files)
    invalid_files.push(("mismatched_01.xml", "<root></mismatch>"));
    invalid_files.push(("mismatched_02.xml", "<root><child></root></child>"));
    invalid_files.push(("mismatched_03.xml", "<Root></root>")); // case mismatch
    invalid_files.push(("mismatched_04.xml", "<root><a><b></a></b></root>"));
    invalid_files.push(("mismatched_05.xml", "<root></>"));

    // Invalid Entities (6 files)
    invalid_files.push(("entity_01.xml", "<root>&invalid;</root>"));
    invalid_files.push(("entity_02.xml", "<root>&;</root>"));
    invalid_files.push(("entity_03.xml", "<root>&#invalid;</root>"));
    invalid_files.push(("entity_04.xml", "<root>&#xinvalid;</root>"));
    invalid_files.push(("entity_05.xml", "<root>&amp</root>")); // truncated entity
    invalid_files.push(("entity_06.xml", "<root>&#1114112;</root>")); // codepoint out of bounds (> 0x10FFFF)

    // Malformed Attributes (6 files)
    invalid_files.push(("attr_01.xml", "<root attr=no-quotes/>"));
    invalid_files.push(("attr_02.xml", "<root attr=\"value'/>")); // mismatched quotes
    invalid_files.push(("attr_03.xml", "<root attr/>")); // no value or =
    invalid_files.push(("attr_04.xml", "<root attr = />")); // empty attribute value
    invalid_files.push(("attr_05.xml", "<root =\"value\"/>")); // missing name
    invalid_files.push(("attr_06.xml", "<root attr1=\"v1\"attr2=\"v2\"/>")); // missing space

    // Depth Exceeded (3 files)
    let deep_101 = build_nested_xml(101);
    invalid_files.push(("depth_101.xml", &deep_101));
    let deep_200 = build_nested_xml(200);
    invalid_files.push(("depth_200.xml", &deep_200));
    let deep_1000 = build_nested_xml(1000);
    invalid_files.push(("depth_1000.xml", &deep_1000));

    // Invalid Characters / Empty (5 files)
    invalid_files.push(("char_01.xml", "")); // empty string
    invalid_files.push(("char_02.xml", "   \n\t ")); // whitespace only
    invalid_files.push(("char_03.xml", "<root>\u{0}</root>")); // null byte
    invalid_files.push(("char_04.xml", "<root>\u{1}\u{2}</root>")); // control chars
    invalid_files.push(("char_05.xml", "<root attr=\"\u{b}\"/>")); // control char in attribute

    for (name, content) in &invalid_files {
        fs::write(invalid_dir.join(name), content).unwrap();
    }

    // 3. Generate Unicode XML Corpus (20+ files)
    let mut unicode_files = Vec::new();

    // CJK (4 files)
    unicode_files.push(("cjk_01.xml", "<中文>中国</中文>"));
    unicode_files.push(("cjk_02.xml", "<日本語 属性=\"値\">日本語テキスト</日本語>"));
    unicode_files.push(("cjk_03.xml", "<한국어>한국어 텍스트</한국어>"));
    unicode_files.push(("cjk_04.xml", "<root cjk=\"繁體字\">简体字</root>"));

    // RTL scripts (4 files)
    unicode_files.push(("rtl_01.xml", "<العربية>نص عربي</العربية>"));
    unicode_files.push(("rtl_02.xml", "<עברית>טקסט בעברית</עברית>"));
    unicode_files.push(("rtl_03.xml", "<root dir=\"rtl\">فارسی</root>"));
    unicode_files.push(("rtl_04.xml", "<root>اردو 123</root>"));

    // Emoji (4 files)
    unicode_files.push(("emoji_01.xml", "<emoji>😀😃😄😁</emoji>"));
    unicode_files.push(("emoji_02.xml", "<root emoji=\"🚀\">skyrocket</root>"));
    unicode_files.push(("emoji_03.xml", "<👉>direction</👉>"));
    unicode_files.push(("emoji_04.xml", "<root>Complex emoji: 👨‍👩‍👧‍👦 family</root>"));

    // Combining characters (4 files)
    unicode_files.push(("combining_01.xml", "<root>áéíóú</root>")); // combining accents
    unicode_files.push(("combining_02.xml", "<root>H͑͗͡e͆͒͒ ͝c͛̕õ͡m͆̕e͆͝s͌͠</root>")); // zalgo
    unicode_files.push(("combining_03.xml", "<root>ñõñ</root>"));
    unicode_files.push(("combining_04.xml", "<root>über</root>"));

    // BMP Bounds / Supplementary Planes (4 files)
    unicode_files.push(("bmp_01.xml", "<root>\u{FFFD} replacement char</root>"));
    unicode_files.push(("bmp_02.xml", "<root>\u{FFFF} max BMP</root>"));
    unicode_files.push(("bmp_03.xml", "<root>\u{10000} Linear A</root>"));
    unicode_files.push(("bmp_04.xml", "<root>\u{10FFFF} Max Unicode</root>"));

    for (name, content) in &unicode_files {
        fs::write(unicode_dir.join(name), content).unwrap();
    }

    println!("Generated XML Corpus successfully!");
}

fn build_nested_xml(depth: usize) -> String {
    let mut open = String::new();
    let mut close = String::new();
    for i in 0..depth {
        open.push_str(&format!("<node_{i}>", i = i));
        close.insert_str(0, &format!("</node_{i}>", i = i));
    }
    format!("{}{}{}", open, "content", close)
}

fn build_wide_xml(width: usize) -> String {
    let mut content = String::new();
    content.push_str("<root>");
    for i in 0..width {
        content.push_str(&format!("<child_{} attr=\"{}\"/>", i, i));
    }
    content.push_str("</root>");
    content
}

fn build_large_xml(elements: usize) -> String {
    let mut content = String::new();
    content.push_str("<root>\n");
    for i in 0..elements {
        content.push_str("  <item id=\"");
        content.push_str(&i.to_string());
        content.push_str("\">\n");
        content.push_str("    <name>Item ");
        content.push_str(&i.to_string());
        content.push_str("</name>\n");
        content.push_str("    <description>This is details for item ");
        content.push_str(&i.to_string());
        content.push_str("</description>\n");
        content.push_str("    <active>true</active>\n");
        content.push_str("  </item>\n");
    }
    content.push_str("</root>");
    content
}
