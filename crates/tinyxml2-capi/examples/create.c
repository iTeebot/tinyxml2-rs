/**
 * create.c — Build an XML document programmatically using the tinyxml2 C API.
 *
 * This example constructs a <config> document containing database connection
 * settings, demonstrates both DOM-based serialisation (tx_document_to_string)
 * and the streaming printer API, then cleans everything up.
 *
 * Build (assuming the shared/static library is available):
 *   cc -I../include -o create create.c -ltinyxml2_capi
 */

#include <stdio.h>
#include <stdlib.h>
#include "../include/tinyxml2.h"

/**
 * Helper: create an element with a single text child and append it to
 * `parent`.  Returns the new element's node ID.
 */
static struct TxNodeId add_text_element(TxDocument *doc,
                                        struct TxNodeId parent,
                                        const char *tag,
                                        const char *text)
{
    struct TxNodeId el   = tx_new_element(doc, tag);
    struct TxNodeId txt  = tx_new_text(doc, text);
    tx_insert_end_child(doc, el, txt);
    tx_insert_end_child(doc, parent, el);
    return el;
}

int main(void)
{
    /* ── 1. Create an empty document ───────────────────────────────── */
    TxDocument *doc = tx_document_new();
    if (!doc) {
        fprintf(stderr, "Error: failed to allocate document\n");
        return EXIT_FAILURE;
    }

    /* ── 2. Build the DOM tree ─────────────────────────────────────── */

    /*  <config version="1.0"> */
    struct TxNodeId root = tx_root_element(doc);
    struct TxNodeId config = tx_new_element(doc, "config");
    tx_element_set_attribute(doc, config, "version", "1.0");

    /* The document node itself is the implicit parent of the root element.
     * We obtain it via tx_parent of any top-level node, but for a fresh
     * document we can use the first-child trick: the document node's ID
     * is always index 0, generation 1.  Instead, we simply insert into
     * the document directly using the document's root convenience. */
    /* Insert <config> as the document's root element. */
    {
        /* The document node is the parent of all top-level nodes.
         * We can obtain it by creating a temporary node and asking for its
         * parent — but the simplest approach is to recognise that the
         * document node is always at index 0.  However, the generation
         * is an implementation detail, so instead we use the fact that
         * tx_root_element returns TX_NULL_NODE for an empty doc and we
         * already have `root` == TX_NULL_NODE.  The correct way is to
         * insert via tx_insert_end_child with the document's own node ID.
         *
         * For tinyxml2-capi the document node ID is internal; the
         * canonical way to attach a root element is:
         */
        struct TxNodeId decl = tx_new_declaration(doc, "xml version=\"1.0\" encoding=\"UTF-8\"");
        /* Insert declaration first (becomes the first child of the
         * document node).  We need the doc-node ID; by convention it is
         * {0, 1} in this implementation, but the stable API approach is
         * to parse a trivial string to get the doc node.  Let's just
         * parse an empty wrapper and clear, or — even simpler — use
         * tx_document_parse to seed the document with a skeleton and
         * then mutate it.  For maximum clarity we'll build everything
         * from scratch using the node IDs directly.
         *
         * The document node is the parent returned for any root-level
         * node.  Since the doc is empty, let's insert `decl` and then
         * ask for its parent. */

        /* Insert the declaration as a child of the document.  Because the
         * document is empty, we first need the document-node ID.  We use
         * a small trick: create a throwaway element, insert it, query its
         * parent (= the document node), then delete it. */
        struct TxNodeId tmp = tx_new_element(doc, "_");
        /* Parse a minimal doc to get the document node. */
        tx_document_parse(doc, "<_/>");
        struct TxNodeId tmp_root = tx_root_element(doc);
        struct TxNodeId doc_node = tx_parent(doc, tmp_root);
        /* Clear and rebuild. */
        tx_document_clear(doc);

        /* Now doc_node may be stale after clear.  Let's use a simpler
         * strategy: parse a declaration + skeleton, then replace the
         * skeleton element. */
        (void)tmp;
        (void)decl;
        (void)doc_node;
    }

    /* ── Simpler approach: parse a minimal skeleton, then mutate ──── */
    tx_document_parse(doc, "<?xml version=\"1.0\" encoding=\"UTF-8\"?><config/>");
    config = tx_root_element(doc);
    tx_element_set_attribute(doc, config, "version", "1.0");

    /*  <database> */
    struct TxNodeId database = tx_new_element(doc, "database");
    tx_element_set_attribute(doc, database, "id", "primary");
    tx_insert_end_child(doc, config, database);

    /*    <host>localhost</host> */
    add_text_element(doc, database, "host", "localhost");

    /*    <port>5432</port> */
    add_text_element(doc, database, "port", "5432");

    /*    <name>myapp_production</name> */
    add_text_element(doc, database, "name", "myapp_production");

    /*    <credentials> */
    struct TxNodeId creds = tx_new_element(doc, "credentials");
    tx_insert_end_child(doc, database, creds);

    /*      <user>admin</user> */
    add_text_element(doc, creds, "user", "admin");

    /*      <password encrypted="true">s3cr3t</password> */
    {
        struct TxNodeId pw = add_text_element(doc, creds, "password", "s3cr3t");
        tx_element_set_attribute(doc, pw, "encrypted", "true");
    }

    /*  <pool> */
    struct TxNodeId pool = tx_new_element(doc, "pool");
    tx_insert_end_child(doc, config, pool);

    add_text_element(doc, pool, "min_connections", "5");
    add_text_element(doc, pool, "max_connections", "20");
    add_text_element(doc, pool, "timeout_seconds", "30");

    /* ── 3. Serialise via tx_document_to_string (pretty) ───────────── */
    printf("── DOM serialisation (pretty-printed) ──\n");
    const char *pretty = tx_document_to_string(doc);
    if (pretty) {
        printf("%s\n", pretty);
    }

    /* ── 4. Serialise via tx_document_to_string_compact ────────────── */
    printf("── DOM serialisation (compact) ──────────\n");
    const char *compact = tx_document_to_string_compact(doc);
    if (compact) {
        printf("%s\n\n", compact);
    }

    /* ── 5. Demonstrate the streaming Printer API ──────────────────── */
    printf("── Printer API output ──────────────────\n");

    TxPrinter *printer = tx_printer_new();  /* pretty-print mode */
    if (!printer) {
        fprintf(stderr, "Error: failed to allocate printer\n");
        tx_document_free(doc);
        return EXIT_FAILURE;
    }

    /* Manually build the same <config> fragment with the printer. */
    tx_printer_open_element(printer, "config");
    tx_printer_push_attribute(printer, "version", "1.0");

    tx_printer_open_element(printer, "database");
    tx_printer_push_attribute(printer, "id", "primary");

    tx_printer_open_element(printer, "host");
    tx_printer_push_text(printer, "localhost");
    tx_printer_close_element(printer);  /* </host> */

    tx_printer_open_element(printer, "port");
    tx_printer_push_text(printer, "5432");
    tx_printer_close_element(printer);  /* </port> */

    tx_printer_open_element(printer, "name");
    tx_printer_push_text(printer, "myapp_production");
    tx_printer_close_element(printer);  /* </name> */

    tx_printer_push_comment(printer, " credentials section ");

    tx_printer_open_element(printer, "credentials");

    tx_printer_open_element(printer, "user");
    tx_printer_push_text(printer, "admin");
    tx_printer_close_element(printer);  /* </user> */

    tx_printer_open_element(printer, "password");
    tx_printer_push_attribute(printer, "encrypted", "true");
    tx_printer_push_text(printer, "s3cr3t");
    tx_printer_close_element(printer);  /* </password> */

    tx_printer_close_element(printer);  /* </credentials> */
    tx_printer_close_element(printer);  /* </database> */

    tx_printer_open_element(printer, "pool");

    tx_printer_open_element(printer, "min_connections");
    tx_printer_push_text(printer, "5");
    tx_printer_close_element(printer);

    tx_printer_open_element(printer, "max_connections");
    tx_printer_push_text(printer, "20");
    tx_printer_close_element(printer);

    tx_printer_open_element(printer, "timeout_seconds");
    tx_printer_push_text(printer, "30");
    tx_printer_close_element(printer);

    tx_printer_close_element(printer);  /* </pool> */
    tx_printer_close_element(printer);  /* </config> */

    const char *printer_out = tx_printer_result(printer);
    if (printer_out) {
        printf("%s\n", printer_out);
    }

    /* ── 6. Clean up ───────────────────────────────────────────────── */
    tx_printer_free(printer);
    tx_document_free(doc);

    printf("\nDone.\n");
    return EXIT_SUCCESS;
}
