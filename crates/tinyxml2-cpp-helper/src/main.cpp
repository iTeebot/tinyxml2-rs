#include <iostream>
#include <string>
#include <vector>
#include "tinyxml2.h"

void escape_json(const char* s, std::ostream& out) {
    if (!s) return;
    for (const char* p = s; *p; ++p) {
        switch (*p) {
            case '\\': out << "\\\\"; break;
            case '"': out << "\\\""; break;
            case '\b': out << "\\b"; break;
            case '\f': out << "\\f"; break;
            case '\n': out << "\\n"; break;
            case '\r': out << "\\r"; break;
            case '\t': out << "\\t"; break;
            default:
                if (static_cast<unsigned char>(*p) < 0x20) {
                    char buf[8];
                    snprintf(buf, sizeof(buf), "\\u%04x", static_cast<unsigned char>(*p));
                    out << buf;
                } else {
                    out << *p;
                }
        }
    }
}

void print_node(const tinyxml2::XMLNode* node) {
    if (!node) return;
    std::cout << "{";
    
    // Type
    std::cout << "\"type\":";
    if (node->ToDocument()) std::cout << "\"document\"";
    else if (node->ToElement()) std::cout << "\"element\"";
    else if (node->ToText()) {
        const tinyxml2::XMLText* text = node->ToText();
        if (text->CData()) std::cout << "\"cdata\"";
        else std::cout << "\"text\"";
    }
    else if (node->ToComment()) std::cout << "\"comment\"";
    else if (node->ToDeclaration()) std::cout << "\"declaration\"";
    else if (node->ToUnknown()) std::cout << "\"unknown\"";
    else std::cout << "\"unknown\"";

    // Value
    std::cout << ",\"value\":";
    if (node->Value()) {
        std::cout << "\"";
        escape_json(node->Value(), std::cout);
        std::cout << "\"";
    } else {
        std::cout << "null";
    }

    // Line number
    std::cout << ",\"line\":" << node->GetLineNum();

    // Attributes (if element)
    if (const tinyxml2::XMLElement* el = node->ToElement()) {
        std::cout << ",\"attributes\":[";
        bool first = true;
        for (const tinyxml2::XMLAttribute* attr = el->FirstAttribute(); attr; attr = attr->Next()) {
            if (!first) std::cout << ",";
            first = false;
            std::cout << "{";
            std::cout << "\"name\":\"";
            escape_json(attr->Name(), std::cout);
            std::cout << "\",\"value\":\"";
            escape_json(attr->Value(), std::cout);
            std::cout << "\"}";
        }
        std::cout << "]";
    }

    // Children
    std::cout << ",\"children\":[";
    bool first_child = true;
    for (const tinyxml2::XMLNode* child = node->FirstChild(); child; child = child->NextSibling()) {
        if (!first_child) std::cout << ",";
        first_child = false;
        print_node(child);
    }
    std::cout << "]";

    std::cout << "}";
}

extern "C" int run_cpp_main(int argc, char** argv) {
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <xml-file-path> [whitespace_mode]" << std::endl;
        return 1;
    }
    
    tinyxml2::Whitespace wsMode = tinyxml2::PRESERVE_WHITESPACE;
    if (argc >= 3) {
        std::string mode(argv[2]);
        if (mode == "collapse") {
            wsMode = tinyxml2::COLLAPSE_WHITESPACE;
        } else if (mode == "pedantic") {
            wsMode = tinyxml2::PEDANTIC_WHITESPACE;
        }
    }
    
    tinyxml2::XMLDocument doc(true, wsMode);
    
    // Default load
    tinyxml2::XMLError err = doc.LoadFile(argv[1]);
    if (err != tinyxml2::XML_SUCCESS) {
        std::cout << "{";
        std::cout << "\"error\":true";
        std::cout << ",\"code\":" << static_cast<int>(err);
        std::cout << ",\"name\":\"" << doc.ErrorIDToName(err) << "\"";
        std::cout << ",\"line\":" << doc.ErrorLineNum();
        std::cout << "}" << std::endl;
        return 0;
    }
    print_node(&doc);
    std::cout << std::endl;
    return 0;
}
