;; title: stacks wars fungible token
;; author: flames.stx
;; version: v1
;; summary: A SIP-010 compliant fungible token for Stacks Wars.
;; description: A fungible token contract implementing the SIP-010 standard,
;;				used for custom token rewards and sponsored lobbies.

;; traits
;;

(impl-trait 'SP3FBR2AGK5H9QBDH3EEN6DF8EK8JY7RX8QJ5SVTE.sip-010-trait-ft-standard.sip-010-trait)

;; token definitions
;;

(define-fungible-token stacks-wars-token)

;; constants
;;
(define-constant CONTRACT-OWNER tx-sender)
(define-constant TOKEN-NAME "Stacks Wars Token")
(define-constant TOKEN-SYMBOL "SWT")
(define-constant TOKEN-DECIMALS u6)

;; error constants
;;
(define-constant ERR-NOT-AUTHORIZED (err u401))
(define-constant ERR-INVALID-AMOUNT (err u402))
(define-constant ERR-INSUFFICIENT-BALANCE (err u403))

;; data vars
;;
(define-data-var token-uri (optional (string-utf8 256)) none)

;; public functions
;;

;; SIP-010 Standard Functions

;; Transfer tokens from sender to recipient
;; @param amount: amount of tokens to transfer
;; @param sender: principal sending the tokens
;; @param recipient: principal receiving the tokens
;; @param memo: optional memo buffer
;; @returns (ok true) on success
(define-public (transfer (amount uint) (sender principal) (recipient principal) (memo (optional (buff 34))))
	(begin
		;; Ensure tx-sender is the sender
		(asserts! (is-eq tx-sender sender) ERR-NOT-AUTHORIZED)

		;; Ensure amount is greater than 0
		(asserts! (> amount u0) ERR-INVALID-AMOUNT)

		;; Transfer tokens
		(try! (ft-transfer? stacks-wars-token amount sender recipient))

		;; Print memo if provided
		(match memo to-print (print to-print) 0x)

		(ok true)
	)
)

;; Get token name
;; @returns (ok TOKEN-NAME)
(define-read-only (get-name)
	(ok "Stacks Wars Token")
)

;; Get token symbol
;; @returns (ok TOKEN-SYMBOL)
(define-read-only (get-symbol)
	(ok "SWT")
)

;; Get token decimals
;; @returns (ok TOKEN-DECIMALS)
(define-read-only (get-decimals)
	(ok TOKEN-DECIMALS)
)

;; Get balance of account
;; @param account: principal to check balance for
;; @returns (ok uint) balance of account
(define-read-only (get-balance (account principal))
	(ok (ft-get-balance stacks-wars-token account))
)

;; Get total supply of tokens
;; @returns (ok uint) total supply
(define-read-only (get-total-supply)
	(ok (ft-get-supply stacks-wars-token))
)

;; Get token URI
;; @returns (ok (optional string-utf8)) token URI
(define-read-only (get-token-uri)
	(ok (var-get token-uri))
)

;; Additional Functions

;; Mint tokens (only contract owner)
;; @param amount: amount of tokens to mint
;; @param recipient: principal receiving the minted tokens
;; @returns (ok true) on success
(define-public (mint (amount uint) (recipient principal))
	(begin
		;; Only contract owner can mint
		(asserts! (is-eq tx-sender CONTRACT-OWNER) ERR-NOT-AUTHORIZED)

		;; Ensure amount is greater than 0
		(asserts! (> amount u0) ERR-INVALID-AMOUNT)

		;; Mint tokens
		(try! (ft-mint? stacks-wars-token amount recipient))

		(ok true)
	)
)

;; Burn tokens
;; @param amount: amount of tokens to burn
;; @returns (ok true) on success
(define-public (burn (amount uint))
	(begin
		;; Ensure amount is greater than 0
		(asserts! (> amount u0) ERR-INVALID-AMOUNT)

		;; Ensure sender has sufficient balance
		(asserts! (>= (ft-get-balance stacks-wars-token tx-sender) amount) ERR-INSUFFICIENT-BALANCE)

		;; Burn tokens
		(try! (ft-burn? stacks-wars-token amount tx-sender))

		(ok true)
	)
)

;; Set token URI (only contract owner)
;; @param uri: new token URI
;; @returns (ok true) on success
(define-public (set-token-uri (uri (string-utf8 256)))
	(begin
		;; Only contract owner can set URI
		(asserts! (is-eq tx-sender CONTRACT-OWNER) ERR-NOT-AUTHORIZED)

		;; Set token URI
		(var-set token-uri (some uri))

		(ok true)
	)
)
