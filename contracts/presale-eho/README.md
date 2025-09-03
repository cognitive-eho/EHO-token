# EHO Presale Contract

## Overview

This repository contains the architectural blueprint for the **\$EHO Presale Contract**. This smart contract is designed to manage the initial token sale for the Cognitive Echo (\$EHO) project on the Neutron blockchain in a secure, transparent, and trustless manner.

The architecture prioritizes clarity, security, and adherence to CosmWasm best practices, ensuring a smooth and reliable fundraising process for both the project team and its early supporters.

---

## Key Features

-   **Whitelist Access Control:** The sale can be restricted to a list of pre-approved addresses, managed by the contract admin.
-   **Configurable Sale Caps:** Both a `soft_cap` (minimum raise for success) and a `hard_cap` (maximum raise) can be defined at launch.
-   **Automated & Trustless Refunds:** If the `soft_cap` is not met by the sale's end, the contract automatically enables a refund mechanism, allowing participants to reclaim their original funds safely.
-   **Secure Admin Controls:** Designated admin functions for starting the sale, managing the whitelist, and withdrawing funds after a successful raise.
-   **Standard CW20 Integration:** Designed to handle payments in a standard CW20 token (USDC in this case) and distribute the `$EHO` CW20 token.
-   **Clear Lifecycle States:** The contract moves through distinct, queryable states (`Pending`, `Active`, `Succeeded`, `Failed`), providing full transparency on the sale's progress.

---

## Contract Architecture

The contract is designed with a clear separation of concerns.

### 1. State (`src/state.rs`)

The contract's "memory" is defined here. It is split into immutable configuration and mutable state.

-   **`Config` (Immutable):** Parameters set once at instantiation that define the rules of the sale.
    -   `admin`: The wallet with administrative privileges.
    -   `eho_token_address`: The address of the `$EHO` token contract.
    -   `usdc_token_address`: The address of the `USDC` token contract used for payment.
    -   `exchange_rate`: The fixed price of `$EHO` in `USDC`.
    -   `start_time` & `end_time`: The sale's active window (as Unix timestamps).
    -   `soft_cap` & `hard_cap`: The minimum and maximum fundraising goals.

-   **`State` (Mutable):** Data that changes as the sale progresses.
    -   `total_usdc_raised`: The current amount of `USDC` collected.
    -   `sale_status`: The current phase of the sale (`Pending`, `Active`, `Succeeded`, or `Failed`).

-   **`Maps` (User Data):**
    -   `CONTRIBUTIONS`: A map storing each user's address and their total `USDC` contribution.
    -   `WHITELIST`: A map storing the addresses authorized to participate.

### 2. Messages (`src/msg.rs`)

This file defines the contract's public API—all the ways one can interact with it.

-   **`InstantiateMsg`**: The message used to deploy and configure the contract with all the `Config` parameters.

-   **`ExecuteMsg` (Actions that change state):**
    -   **User Actions:**
        -   `Receive(Cw20ReceiveMsg)`: The standard way for a user to send `USDC` to the contract and "buy" their allocation.
        -   `ClaimTokens {}`: Called by a user after a *successful* sale to receive their purchased `$EHO` tokens.
        -   `RequestRefund {}`: Called by a user after a *failed* sale to reclaim their invested `USDC`.
    -   **Admin Actions:**
        -   `StartSale {}`: Moves the contract state from `Pending` to `Active`.
        -   `AddToWhitelist { addresses }`: Adds new participants to the sale.
        -   `WithdrawFunds {}`: Called by the admin after a *successful* sale to transfer the raised `USDC` to the treasury.

-   **`QueryMsg` (Actions that read state):**
    -   `Config {}`: Returns the contract's immutable configuration.
    -   `State {}`: Returns the current sale status and total amount raised.
    -   `IsWhitelisted { address }`: Checks if a specific address is allowed to participate.
    -   `ContributionOf { address }`: Returns how much a specific user has contributed.

### 3. Error Handling (`src/error.rs`)

To provide maximum clarity, the contract uses a set of custom errors. Instead of a generic failure, users and developers will see specific reasons for failed transactions, such as:
-   `Unauthorized`
-   `SaleNotActive`
-   `HardCapReached`
-   `NotInWhitelist`
-   `SoftCapNotReached`

---

## Presale Lifecycle

The contract follows a strict and predictable lifecycle:

1.  **Setup:** The contract is deployed (`instantiate`) with all rules defined. The project team then transfers the full amount of `$EHO` for sale (e.g., 400M tokens) directly to this presale contract address.

2.  **Pending:** Before the `start_time`, the contract is in a `Pending` state. The admin can manage the whitelist during this period. No contributions are accepted.

3.  **Active:** Once the `start_time` is reached (and the admin calls `StartSale`), the state becomes `Active`. Whitelisted users can now send `USDC` to the contract to buy `$EHO`. This continues until the `hard_cap` is reached or the `end_time` passes.

4.  **End of Sale:** The sale concludes. The contract checks if `total_usdc_raised` is greater than or equal to the `soft_cap`.

5.  **Path A: Success (`Succeeded` state):**
    -   Participants can now call `ClaimTokens` to receive their `$EHO`.
    -   The admin can call `WithdrawFunds` to move the collected `USDC` to the project treasury.
    -   The `RequestRefund` function is disabled.

6.  **Path B: Failure (`Failed` state):**
    -   Participants can now call `RequestRefund` to get their original `USDC` back.
    -   The `ClaimTokens` and `WithdrawFunds` functions are permanently disabled.
    -   The admin can reclaim the unsold `$EHO` tokens from the contract.

---

## Development & Usage

This is an architectural blueprint. The core logic is to be implemented in the next milestone.

### Key Commands
-   **Run Tests:** `cargo test --workspace`
-   **Generate JSON Schema:** `cargo run --example schema`
-   **Compile for Production:** `cargo run-script optimize`

### Current Status
-   [x] State definition complete.
-   [x] Message (API) definition complete.
-   [x] Error handling structure complete.
-   [ ] Core logic implementation and unit tests.

This architecture provides a secure foundation for the $EHO presale contract.