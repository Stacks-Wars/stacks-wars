;; title: stacks wars stx vault
;; author: flames.stx
;; version: v1
;; summary:
;; description:

;; traits
;;

;; token definitions
;;

;; constants
(define-constant ENTRY-FEE u5000000) ;; 5 STX in microSTX
(define-constant DEPLOYER 'ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM)

;; error constants
(define-constant ERR-ALREADY-JOINED (err u100))
(define-constant ERR-DEPLOYER-MUST-JOIN-FIRST (err u101))
(define-constant ERR-NOT-DEPLOYER (err u102))
(define-constant ERR-CANNOT-KICK-SELF (err u103))

;; data vars
(define-data-var total-players uint u0)

;; data maps
(define-map players principal uint)

;; public functions

;; Join the lobby by depositing the entry fee
;; @returns (ok true) on success
(define-public (join)
	(let
		(
			(sender tx-sender)
			(player-count (var-get total-players))
		)
		;; Check if player has already joined
		(asserts! (is-none (map-get? players sender)) ERR-ALREADY-JOINED)

		;; Ensure deployer joins first (if count is 0, sender must be deployer)
		(asserts!
			(or (> player-count u0) (is-eq sender DEPLOYER))
			ERR-DEPLOYER-MUST-JOIN-FIRST
		)

		;; Transfer entry fee from sender to contract
		(try! (stx-transfer? ENTRY-FEE sender (as-contract tx-sender)))

		;; Update state
		(map-insert players sender stacks-block-height)
		(var-set total-players (+ player-count u1))

		(ok true)
	)
)

;; Leave the lobby and withdraw deposit
;; @returns (ok true) on success
(define-public (leave)
	(ok true)
)

;; Claim reward after game completion
;; @param recipient: principal address to receive the reward
;; @param amount: reward amount in microSTX
;; @returns (ok true) on success
(define-public (claim-reward (recipient principal) (amount uint))
	(ok true)
)

;; Kick a player from the lobby (creator only, before game starts)
;; @param player: principal address of player to kick
;; @returns (ok true) on success
(define-public (kick (player principal))
	(let
		(
			(sender tx-sender)
			(player-count (var-get total-players))
		)
		;; Only deployer can kick
		(asserts! (is-eq sender DEPLOYER) ERR-NOT-DEPLOYER)

		;; Check if player is in the lobby
		(asserts! (is-some (map-get? players player)) (ok false))

		;; Deployer cannot kick themselves
		(asserts! (not (is-eq player DEPLOYER)) ERR-CANNOT-KICK-SELF)

		;; Transfer entry fee from contract back to player
		(try! (as-contract (stx-transfer? ENTRY-FEE tx-sender player)))

		;; Update state
		(map-delete players player)
		(var-set total-players (- player-count u1))

		(ok true)
	)
)

;; read only functions

;; Get the total number of players in the vault
;; @returns uint: total player count
(define-read-only (get-total-players)
	(var-get total-players)
)

;; Check if a player has joined
;; @param player: principal to check
;; @returns bool: true if player has joined
(define-read-only (has-joined (player principal))
	(is-some (map-get? players player))
)

;; private functions
;;

