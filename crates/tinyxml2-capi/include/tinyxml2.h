#ifndef TINYXML2_RS_H
#define TINYXML2_RS_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * C-compatible error code enum matching `TinyXML2` error values.
 */
typedef enum TxError {
    /**
     * Operation completed successfully.
     */
    TxSuccess = 0,
    /**
     * The document was empty or contained no XML content.
     */
    TxErrorEmptyDocument = 1,
    /**
     * Error while parsing an element.
     */
    TxErrorParsingElement = 2,
    /**
     * Error while parsing an attribute.
     */
    TxErrorParsingAttribute = 3,
    /**
     * Error while parsing text content.
     */
    TxErrorParsingText = 4,
    /**
     * Error while parsing a CDATA section.
     */
    TxErrorParsingCdata = 5,
    /**
     * Error while parsing a comment.
     */
    TxErrorParsingComment = 6,
    /**
     * Error while parsing a declaration.
     */
    TxErrorParsingDeclaration = 7,
    /**
     * Error while parsing an unknown construct.
     */
    TxErrorParsingUnknown = 8,
    /**
     * An element's closing tag did not match its opening tag.
     */
    TxErrorMismatchedElement = 9,
    /**
     * The specified file was not found.
     */
    TxErrorFileNotFound = 10,
    /**
     * An error occurred while reading/writing a file.
     */
    TxErrorFileRead = 11,
    /**
     * The maximum element nesting depth was exceeded.
     */
    TxErrorElementDepthExceeded = 12,
    /**
     * The requested attribute does not exist.
     */
    TxErrorNoAttribute = 13,
    /**
     * The attribute value could not be converted to the requested type.
     */
    TxErrorWrongAttributeType = 14,
    /**
     * Text content could not be converted to the requested type.
     */
    TxErrorCanNotConvertText = 15,
    /**
     * No text node was found as a child of the element.
     */
    TxErrorNoTextNode = 16,
    /**
     * The provided node ID is invalid or refers to a freed node.
     */
    TxErrorInvalidNodeId = 17,
} TxError;

/**
 * C-compatible representation of the XML node type.
 */
typedef enum TxNodeType {
    /**
     * The document root node.
     */
    TxNodeDocument = 0,
    /**
     * An XML element (tag).
     */
    TxNodeElement = 1,
    /**
     * A text content node.
     */
    TxNodeText = 2,
    /**
     * An XML comment (`<!-- ... -->`).
     */
    TxNodeComment = 3,
    /**
     * An XML declaration (`<?xml ... ?>`).
     */
    TxNodeDeclaration = 4,
    /**
     * An unrecognized XML construct.
     */
    TxNodeUnknown = 5,
} TxNodeType;

/**
 * A C-compatible node identifier.
 *
 * This is a `#[repr(C)]` mirror of the internal `NodeId` type. It carries
 * both an arena index and a generation counter for use-after-free detection.
 */
typedef struct TxNodeId {
    /**
     * Index into the document's node arena.
     */
    uint32_t index;
    /**
     * Generation counter for validity checking.
     */
    uint32_t generation;
} TxNodeId;

/**
 * Sentinel value representing a null / invalid node.
 *
 * Any function that would return "no node" returns this sentinel.
 * Use [`tx_node_is_null`](crate::tx_node_is_null) to test for it.
 */
#define TX_NULL_NODE (TxNodeId){ .index = UINT32_MAX, .generation = 0 }

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Creates a new, empty XML document.
 *
 * The returned pointer must eventually be freed with [`tx_document_free`].
 *
 * # Safety
 *
 * The caller must free the returned pointer with [`tx_document_free`] when
 * done. Returns null on allocation failure.
 */
TxDocument *tx_document_new(void);

/**
 * Frees a document previously created with [`tx_document_new`].
 *
 * # Safety
 *
 * `doc` must be a valid pointer returned by [`tx_document_new`], or null.
 * After this call, the pointer must not be used again.
 */
void tx_document_free(TxDocument *doc);

/**
 * Clears the document, removing all nodes and resetting to an empty state.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
void tx_document_clear(TxDocument *doc);

/**
 * Parses an XML string into the document, replacing any existing content.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `xml` must be a valid, non-null pointer to a null-terminated UTF-8 C string.
 */
enum TxError tx_document_parse(TxDocument *doc, const char *xml);

/**
 * Loads and parses an XML file.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `path` must be a valid, non-null pointer to a null-terminated UTF-8 C string
 *   containing a filesystem path.
 */
enum TxError tx_document_load_file(TxDocument *doc, const char *path);

/**
 * Saves the document to a file (pretty-printed).
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `path` must be a valid, non-null pointer to a null-terminated UTF-8 C string
 *   containing a filesystem path.
 */
enum TxError tx_document_save_file(const TxDocument *doc, const char *path);

/**
 * Returns the pretty-printed XML string for the document.
 *
 * The returned pointer is valid until the next mutating operation on the
 * document or until the document is freed.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
const char *tx_document_to_string(TxDocument *doc);

/**
 * Returns the compact XML string for the document.
 *
 * The returned pointer is valid until the next mutating operation on the
 * document or until the document is freed.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
const char *tx_document_to_string_compact(TxDocument *doc);

/**
 * Returns the current error code of the document.
 *
 * Returns [`TxError::TxSuccess`] if no error has occurred.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
enum TxError tx_document_error(const TxDocument *doc);

/**
 * Returns the line number of the current error, or 0 if no error.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
int tx_document_error_line(const TxDocument *doc);

/**
 * Returns the error name string, or null if no error.
 *
 * The returned pointer is valid until the next mutating operation on the
 * document or until the document is freed.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
const char *tx_document_error_name(TxDocument *doc);

/**
 * Creates a new element node with the given tag name.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
struct TxNodeId tx_new_element(TxDocument *doc, const char *name);

/**
 * Creates a new text node with the given content.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
struct TxNodeId tx_new_text(TxDocument *doc, const char *text);

/**
 * Creates a new comment node.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
struct TxNodeId tx_new_comment(TxDocument *doc, const char *text);

/**
 * Creates a new declaration node.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
struct TxNodeId tx_new_declaration(TxDocument *doc, const char *text);

/**
 * Creates a new "unknown" node.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
struct TxNodeId tx_new_unknown(TxDocument *doc, const char *text);

/**
 * Inserts `child` as the last child of `parent`.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
enum TxError tx_insert_end_child(TxDocument *doc, struct TxNodeId parent, struct TxNodeId child);

/**
 * Inserts `child` as the first child of `parent`.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
enum TxError tx_insert_first_child(TxDocument *doc, struct TxNodeId parent, struct TxNodeId child);

/**
 * Inserts `child` as the next sibling after `after`.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
enum TxError tx_insert_after_child(TxDocument *doc, struct TxNodeId after, struct TxNodeId child);

/**
 * Deletes `child` from `parent`.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
enum TxError tx_delete_child(TxDocument *doc, struct TxNodeId parent, struct TxNodeId child);

/**
 * Deletes all children of `parent`.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
enum TxError tx_delete_children(TxDocument *doc, struct TxNodeId parent);

/**
 * Deletes a node and all its descendants from the document.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
enum TxError tx_delete_node(TxDocument *doc, struct TxNodeId node);

/**
 * Returns the parent of the given node, or `TX_NULL_NODE` if none.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
struct TxNodeId tx_parent(const TxDocument *doc, struct TxNodeId node);

/**
 * Returns the first child of the given node, or `TX_NULL_NODE` if none.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
struct TxNodeId tx_first_child(const TxDocument *doc, struct TxNodeId node);

/**
 * Returns the last child of the given node, or `TX_NULL_NODE` if none.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
struct TxNodeId tx_last_child(const TxDocument *doc, struct TxNodeId node);

/**
 * Returns the previous sibling of the given node, or `TX_NULL_NODE` if none.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
struct TxNodeId tx_prev_sibling(const TxDocument *doc, struct TxNodeId node);

/**
 * Returns the next sibling of the given node, or `TX_NULL_NODE` if none.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
struct TxNodeId tx_next_sibling(const TxDocument *doc, struct TxNodeId node);

/**
 * Returns the first child element, optionally filtered by tag name.
 *
 * If `name` is null, returns the first child element regardless of its name.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name`, if non-null, must be a valid null-terminated UTF-8 string.
 */
struct TxNodeId tx_first_child_element(const TxDocument *doc,
                                       struct TxNodeId node,
                                       const char *name);

/**
 * Returns the next sibling element, optionally filtered by tag name.
 *
 * If `name` is null, returns the next sibling element regardless of its name.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name`, if non-null, must be a valid null-terminated UTF-8 string.
 */
struct TxNodeId tx_next_sibling_element(const TxDocument *doc,
                                        struct TxNodeId node,
                                        const char *name);

/**
 * Returns the root element of the document, or `TX_NULL_NODE` if the
 * document is empty.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
struct TxNodeId tx_root_element(const TxDocument *doc);

/**
 * Returns the tag name of an element node.
 *
 * The returned pointer is valid until the next mutating operation on the
 * document or until the document is freed.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `element` must identify an element node.
 */
const char *tx_element_name(TxDocument *doc, struct TxNodeId element);

/**
 * Returns the value of the named attribute on an element.
 *
 * Returns null if the element or attribute does not exist.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
const char *tx_element_attribute(TxDocument *doc, struct TxNodeId el, const char *name);

/**
 * Sets an attribute on an element. Creates the attribute if it doesn't exist,
 * or updates its value if it does.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name` and `value` must be valid, non-null pointers to null-terminated
 *   UTF-8 strings.
 */
enum TxError tx_element_set_attribute(TxDocument *doc,
                                      struct TxNodeId el,
                                      const char *name,
                                      const char *value);

/**
 * Deletes an attribute from an element by name.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
enum TxError tx_element_delete_attribute(TxDocument *doc, struct TxNodeId el, const char *name);

/**
 * Returns the text content of an element's first child text node.
 *
 * Returns null if no text child exists.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `element` must identify an element node.
 */
const char *tx_element_get_text(TxDocument *doc, struct TxNodeId element);

/**
 * Sets the text content of an element.
 *
 * If a child text node exists, its content is replaced. Otherwise, a new
 * text node is created and inserted as the first child.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
enum TxError tx_element_set_text(TxDocument *doc, struct TxNodeId element, const char *text);

/**
 * Queries an integer attribute value.
 *
 * On success, writes the value to `*value` and returns `TxSuccess`.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 * - `value` must be a valid, non-null pointer to a `c_int`.
 */
enum TxError tx_query_int_attribute(const TxDocument *doc,
                                    struct TxNodeId el,
                                    const char *name,
                                    int *value);

/**
 * Queries a boolean attribute value.
 *
 * On success, writes the value to `*value` and returns `TxSuccess`.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 * - `value` must be a valid, non-null pointer to a `bool`.
 */
enum TxError tx_query_bool_attribute(const TxDocument *doc,
                                     struct TxNodeId el,
                                     const char *name,
                                     bool *value);

/**
 * Queries a double (f64) attribute value.
 *
 * On success, writes the value to `*value` and returns `TxSuccess`.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 * - `value` must be a valid, non-null pointer to a `c_double`.
 */
enum TxError tx_query_double_attribute(const TxDocument *doc,
                                       struct TxNodeId el,
                                       const char *name,
                                       double *value);

/**
 * Returns an integer attribute value, or `default_val` if the attribute
 * does not exist or cannot be parsed.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
int tx_int_attribute(const TxDocument *doc, struct TxNodeId el, const char *name, int default_val);

/**
 * Returns a boolean attribute value, or `default_val` if the attribute
 * does not exist or cannot be parsed.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
bool tx_bool_attribute(const TxDocument *doc,
                       struct TxNodeId el,
                       const char *name,
                       bool default_val);

/**
 * Returns a double (f64) attribute value, or `default_val` if the attribute
 * does not exist or cannot be parsed.
 *
 * # Safety
 *
 * - `doc` must be a valid, non-null pointer to a `TxDocument`.
 * - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
double tx_double_attribute(const TxDocument *doc,
                           struct TxNodeId el,
                           const char *name,
                           double default_val);

/**
 * Creates a new XML printer (pretty-print mode).
 *
 * # Safety
 *
 * The returned pointer must eventually be freed with [`tx_printer_free`].
 */
TxPrinter *tx_printer_new(void);

/**
 * Creates a new XML printer (compact mode, no whitespace).
 *
 * # Safety
 *
 * The returned pointer must eventually be freed with [`tx_printer_free`].
 */
TxPrinter *tx_printer_new_compact(void);

/**
 * Frees a printer previously created with [`tx_printer_new`] or
 * [`tx_printer_new_compact`].
 *
 * # Safety
 *
 * `printer` must be a valid pointer returned by a printer constructor, or null.
 * After this call, the pointer must not be used again.
 */
void tx_printer_free(TxPrinter *printer);

/**
 * Opens an element tag in the printer output.
 *
 * # Safety
 *
 * - `printer` must be a valid, non-null pointer to a `TxPrinter`.
 * - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
void tx_printer_open_element(TxPrinter *printer, const char *name);

/**
 * Pushes an attribute onto the currently open element.
 *
 * # Safety
 *
 * - `printer` must be a valid, non-null pointer to a `TxPrinter`.
 * - `name` and `value` must be valid, non-null pointers to null-terminated
 *   UTF-8 strings.
 */
void tx_printer_push_attribute(TxPrinter *printer, const char *name, const char *value);

/**
 * Closes the most recently opened element.
 *
 * # Safety
 *
 * `printer` must be a valid, non-null pointer to a `TxPrinter`.
 */
void tx_printer_close_element(TxPrinter *printer);

/**
 * Pushes text content into the current element.
 *
 * # Safety
 *
 * - `printer` must be a valid, non-null pointer to a `TxPrinter`.
 * - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
void tx_printer_push_text(TxPrinter *printer, const char *text);

/**
 * Pushes a comment into the printer output.
 *
 * # Safety
 *
 * - `printer` must be a valid, non-null pointer to a `TxPrinter`.
 * - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
 */
void tx_printer_push_comment(TxPrinter *printer, const char *text);

/**
 * Returns the accumulated printer output as a C string.
 *
 * The returned pointer is valid until the printer is modified or freed.
 *
 * # Safety
 *
 * `printer` must be a valid, non-null pointer to a `TxPrinter`.
 */
const char *tx_printer_result(TxPrinter *printer);

/**
 * Clears the printer output, resetting it to empty.
 *
 * # Safety
 *
 * `printer` must be a valid, non-null pointer to a `TxPrinter`.
 */
void tx_printer_clear(TxPrinter *printer);

/**
 * Returns the type of the given node.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
enum TxNodeType tx_node_type(const TxDocument *doc, struct TxNodeId node);

/**
 * Returns `true` if the given node ID is the null sentinel.
 *
 * This function is safe to call without a document pointer.
 */
bool tx_node_is_null(struct TxNodeId node);

/**
 * Returns the "value" of a node based on its type:
 *
 * - Element → tag name
 * - Text → text content
 * - Comment → comment text
 * - Declaration → declaration name
 * - Unknown → raw content
 * - Document → empty string
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
const char *tx_node_value(TxDocument *doc, struct TxNodeId node);

/**
 * Returns the 1-based source line number where the node was parsed,
 * or 0 if the node was not created by parsing.
 *
 * # Safety
 *
 * `doc` must be a valid, non-null pointer to a `TxDocument`.
 */
int tx_node_line(const TxDocument *doc, struct TxNodeId node);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* TINYXML2_RS_H */
