# Betting Contract

This contract facilitates betting functionalities within a blockchain environment. Users can initialize a bet, register participants, buy tickets, and play the bet. Below is a breakdown of the contract functionalities:

## Initialization
- **init_bet**: Initializes the bet with parameters such as the admin address, token address, maximum winners count, and ticket price.

## Participant Management
- **register**: Registers a participant in the bet.
- **buy_ticket**: Allows participants to buy tickets for the bet.

## Bet Execution
- **play_bet**: Executes the bet by selecting winners based on a random seed and distributing payouts accordingly.

## Error Handling
The contract defines several error conditions to handle various scenarios such as insufficient funds, already initialized bet, already played bet, and more.

## Storage Keys
- **Admin**: Stores the address of the admin.
- **Candidates**: Stores the list of participant addresses.
- **MaxWinnerCount**: Stores the maximum count of winners allowed.
- **TicketPrice**: Stores the price of a ticket.
- **Token**: Stores the address of the token.
- **AlreadyInitialized**: Indicates whether the bet has already been initialized.
- **AlreadyPlayed**: Indicates whether the bet has already been played.

## Error Types
The contract defines custom error types to provide clear feedback in case of failures during execution.
