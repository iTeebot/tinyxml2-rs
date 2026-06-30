#include "tinyxml2.h"
#include <stdlib.h>
#include <string.h>

extern "C" {
    void* cpp_parse(const char* xml) {
        tinyxml2::XMLDocument* doc = new tinyxml2::XMLDocument();
        if (doc->Parse(xml) == tinyxml2::XML_SUCCESS) {
            return doc;
        }
        delete doc;
        return nullptr;
    }

    void cpp_free(void* doc) {
        delete static_cast<tinyxml2::XMLDocument*>(doc);
    }

    void* cpp_print_compact(void* doc, size_t* out_len) {
        tinyxml2::XMLPrinter printer(nullptr, true);
        static_cast<tinyxml2::XMLDocument*>(doc)->Accept(&printer);
        const char* cstr = printer.CStr();
        size_t len = printer.CStrSize() - 1;
        char* copy = (char*)malloc(len + 1);
        if (copy) {
            memcpy(copy, cstr, len + 1);
        }
        *out_len = len;
        return copy;
    }

    void* cpp_print_pretty(void* doc, size_t* out_len) {
        tinyxml2::XMLPrinter printer;
        static_cast<tinyxml2::XMLDocument*>(doc)->Accept(&printer);
        const char* cstr = printer.CStr();
        size_t len = printer.CStrSize() - 1;
        char* copy = (char*)malloc(len + 1);
        if (copy) {
            memcpy(copy, cstr, len + 1);
        }
        *out_len = len;
        return copy;
    }

    void cpp_free_str(void* str) {
        free(str);
    }
}
