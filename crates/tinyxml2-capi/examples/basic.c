/**
 * basic.c — Parse and traverse an XML document using the tinyxml2 C API.
 *
 * This example parses a small bookstore catalog, iterates over every <book>
 * element, and prints each book's category attribute, title, author, year,
 * and price.
 *
 * Build (assuming the shared/static library is available):
 *   cc -I../include -o basic basic.c -ltinyxml2_capi
 */

#include <stdio.h>
#include <stdlib.h>
#include "../include/tinyxml2.h"

/* A tiny bookstore catalog used as sample input. */
static const char *BOOKSTORE_XML =
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
    "<bookstore>"
    "  <book category=\"fiction\">"
    "    <title lang=\"en\">The Great Gatsby</title>"
    "    <author>F. Scott Fitzgerald</author>"
    "    <year>1925</year>"
    "    <price>10.99</price>"
    "  </book>"
    "  <book category=\"science\">"
    "    <title lang=\"en\">A Brief History of Time</title>"
    "    <author>Stephen Hawking</author>"
    "    <year>1988</year>"
    "    <price>15.50</price>"
    "  </book>"
    "  <book category=\"fiction\">"
    "    <title lang=\"es\">Cien años de soledad</title>"
    "    <author>Gabriel García Márquez</author>"
    "    <year>1967</year>"
    "    <price>12.00</price>"
    "  </book>"
    "</bookstore>";

int main(void)
{
    /* ── 1. Create a document and parse the XML string ─────────────── */
    TxDocument *doc = tx_document_new();
    if (!doc) {
        fprintf(stderr, "Error: failed to allocate document\n");
        return EXIT_FAILURE;
    }

    enum TxError err = tx_document_parse(doc, BOOKSTORE_XML);
    if (err != TxSuccess) {
        fprintf(stderr, "Parse error %d at line %d: %s\n",
                (int)err,
                tx_document_error_line(doc),
                tx_document_error_name(doc));
        tx_document_free(doc);
        return EXIT_FAILURE;
    }

    /* ── 2. Get the root element (<bookstore>) ─────────────────────── */
    struct TxNodeId root = tx_root_element(doc);
    if (tx_node_is_null(root)) {
        fprintf(stderr, "Error: document has no root element\n");
        tx_document_free(doc);
        return EXIT_FAILURE;
    }

    const char *root_name = tx_element_name(doc, root);
    printf("Root element: <%s>\n\n", root_name ? root_name : "(null)");

    /* ── 3. Iterate over <book> children ───────────────────────────── */
    int book_num = 0;
    struct TxNodeId book = tx_first_child_element(doc, root, "book");

    while (!tx_node_is_null(book)) {
        book_num++;
        printf("── Book %d ──────────────────────────\n", book_num);

        /* Read the "category" attribute. */
        const char *category = tx_element_attribute(doc, book, "category");
        printf("  Category : %s\n", category ? category : "(none)");

        /* <title> — also read its "lang" attribute. */
        struct TxNodeId title = tx_first_child_element(doc, book, "title");
        if (!tx_node_is_null(title)) {
            const char *title_text = tx_element_get_text(doc, title);
            const char *lang       = tx_element_attribute(doc, title, "lang");
            printf("  Title    : %s", title_text ? title_text : "(none)");
            if (lang) {
                printf("  [lang=%s]", lang);
            }
            printf("\n");
        }

        /* <author> */
        struct TxNodeId author = tx_first_child_element(doc, book, "author");
        if (!tx_node_is_null(author)) {
            const char *author_text = tx_element_get_text(doc, author);
            printf("  Author   : %s\n", author_text ? author_text : "(none)");
        }

        /* <year> */
        struct TxNodeId year = tx_first_child_element(doc, book, "year");
        if (!tx_node_is_null(year)) {
            const char *year_text = tx_element_get_text(doc, year);
            printf("  Year     : %s\n", year_text ? year_text : "(none)");
        }

        /* <price> */
        struct TxNodeId price = tx_first_child_element(doc, book, "price");
        if (!tx_node_is_null(price)) {
            const char *price_text = tx_element_get_text(doc, price);
            printf("  Price    : $%s\n", price_text ? price_text : "?");
        }

        printf("\n");

        /* Advance to the next <book> sibling. */
        book = tx_next_sibling_element(doc, book, "book");
    }

    printf("Total books: %d\n", book_num);

    /* ── 4. Print the full pretty-printed document ─────────────────── */
    printf("\n── Full document ────────────────────\n");
    const char *xml_out = tx_document_to_string(doc);
    if (xml_out) {
        printf("%s\n", xml_out);
    }

    /* ── 5. Clean up ───────────────────────────────────────────────── */
    tx_document_free(doc);
    return EXIT_SUCCESS;
}
