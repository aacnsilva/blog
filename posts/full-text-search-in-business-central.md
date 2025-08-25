---
title: Full-text search in Business Central
date: 2025-07-19
---

Effective search functionality is crucial for navigating large datasets and finding the information you need quickly within your business applications. With the release of **Business Central 25**, there's an enhanced approach to text search that significantly impacts how you retrieve precise results. Let's explore the power of full-text search within Business Central and what sets it apart.

## How it works

Full-text search in Business Central is designed for intelligent and accurate retrieval of information within text fields. It goes beyond simple string matching by understanding words and their boundaries, offering a more refined search experience than previous versions.

### The Search Syntax

In Business Central, full-text search often utilizes specific syntax to trigger its optimized behavior. When a search is performed on fields specifically designed for full-text capabilities, the system applies these advanced filtering rules. If a field isn't optimized, the system will fall back to legacy filtering to ensure a result is still provided.

Here’s a comparison of how different search inputs might be interpreted:

| Full-text search input | Equivalent traditional search logic     | Microsoft fall-back |
| :--------------------- | :-------------------------------------- | :------------------ |
| `&&com`                | @ \*COM\*                                | @\*COM\* 
| `&&com&&&cust`         | @ \*COM\*& \*CUST\*                       | @\*COM\*&\*CUST\* 

#### Note
Please take into consideration the added space in the equivalent search logic, as this is what mimics the full-text search results

### Precision in Results: A Key Difference

A primary advantage of full-text search in Business Central is its ability to deliver more focused and relevant results. Unlike searches that might capture any occurrence of a string, full-text search prioritizes matching whole words or the beginnings of words.

Consider these records: `ECOMMERCE CUSTOMER`, `COMPANY CUSTOMER`, `ADATUM COMPANY`

Let's see how different search types might perform:

**Scenario 1: Searching for "com"**

*   **Full-text search:** Retrieves `COMPANY CUSTOMER`, `ADATUM COMPANY`
    *   (It matches the word "COMPANY" in both records, as "COMPANY" starts with "com")
*   **Traditional substring search:** Retrieves `ECOMMERCE CUSTOMER`, `COMPANY CUSTOMER`, `ADATUM COMPANY`
    *   (It finds "COMMERCE" within `ECOMMERCE CUSTOMER`, and "COMPANY" in the other two, capturing any occurrence of "com")

**Scenario 2: Searching for "com" AND "cust"**

*   **Full-text search:** Retrieves `COMPANY CUSTOMER`
    *   (It finds records containing both a word starting with "com" AND a word starting with "cust")
*   **Traditional substring search:** Retrieves `ECOMMERCE CUSTOMER`, `COMPANY CUSTOMER`
    *   (It finds records containing "com" anywhere AND "cust" anywhere)

This demonstrates that full-text search provides a more refined result set by intelligently focusing on word beginnings and meaningful word units.

## Understanding Optimized Text Search in Business Central

Microsoft's documentation, which applies to this functionality in Business Central, further clarifies its capabilities compared to traditional wildcard searches:

*   **Case Insensitivity:** Optimized text search is inherently case-insensitive, meaning "apple" will match "Apple" or "APPLE" without special handling. Traditional wildcard searches often require explicit settings or operators to achieve case insensitivity.
*   **Accent Insensitivity:** Similarly, optimized text search is accent-insensitive, so "résumé" will match "resume." Wildcard searches typically treat accented and unaccented characters as distinct unless specific collation settings are applied.
*   **Word-Based Matching:** This is a crucial distinction:
    *   **Optimized text search** looks for **words or prefixes of words** within fields. For example, searching for "app" would find "application" or "apple".
    *   **Traditional wildcard search** can find **arbitrary substrings** within words. A wildcard search for `*app*` would find "unhappy" because "app" is a substring, even if it's not the start of a word.

For a deeper dive into these technical aspects, especially in the context of Business Central development, you can consult resources like the Microsoft documentation on table field text search: [https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-table-field-text-search](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-table-field-text-search)

## Conclusion

With the release of Business Central, full-text search offers a sophisticated and precise method for data retrieval. By understanding its word-based approach, inherent case and accent insensitivity, and how it differs from traditional substring matching, you can leverage this powerful feature to find exactly what you're looking for with greater efficiency and accuracy within your Business Central environment.
