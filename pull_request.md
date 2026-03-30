# Pull Request: Soroban Flash Loan Provider Implementation

## Description
This PR implements a basic **Flash Loan Provider** smart contract for Soroban tokens, as requested in issue #22. 

The contract allows borrowers to take out instantaneous loans of any supported Soroban token, provided they return the principal plus a fixed fee within the same transaction.

## Key Features
- **Flash Loan Mechanism**: Implements `flash_loan(receiver, token, amount)` which handles the funds transfer and external contract invocation.
- **Repayment Enforcement**: Atomic balance verification ensures that the contract's balance is restored with the required fee (0.05%) by the end of the execution logic.
- **Event Emission**: Publishes a `flash_ln` event upon every successful loan execution for transparency and tracking.
- **Unit Testing**: Includes a comprehensive test suite (`tests.rs`) with mock receivers to validate both successful repayments and failed (reverted) loans.

## Implementation Details
- **Location**: `src/flash_loan/`
- **Fee Structure**: Fixed at 5 basis points (0.05%).
- **Receiver Interface**: Borrowers must implement the `FlashLoanReceiver` trait's `execute_loan` method.

## Verification
I have manually verified the implementation logic against Soroban SDK standards and existing patterns in the `AnchorPoint` repository. The code structures and dependency management match the project's architecture.

Fixes #22
