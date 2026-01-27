;; title: stacks wars sponsored ft vault
;; author: flames.stx
;; version: v1
;; summary: A sponsored fungible token vault contract for Stacks Wars.
;; description: A sponsored fungible token vault contract for Stacks Wars where the
;;				deployer funds the entire prize pool. Players join for free,
;;				and the deployer deposits the pool amount when joining first.

;; traits
;;

(use-trait ft-trait 'SP3FBR2AGK5H9QBDH3EEN6DF8EK8JY7RX8QJ5SVTE.sip-010-trait-ft-standard.sip-010-trait)

;; constants
;;

(define-constant POOL-SIZE u10000000)
(define-constant DEPLOYER tx-sender)
(define-constant TRUSTED-PUBLIC-KEY 0x0390a5cac7c33fda49f70bc1b0866fa0ba7a9440d9de647fecb8132ceb76a94dfa)
(define-constant FEE-WALLET 'ST1SJ3DTE5DN7X54YDH5D64R3BCB6A2AG2ZQ8YPD5)
(define-constant FEE-PERCENTAGE u2)

;; error constants
;;

(define-constant ERR-ALREADY-JOINED (err u100))
(define-constant ERR-DEPLOYER-MUST-JOIN-FIRST (err u101))
(define-constant ERR-NOT-DEPLOYER (err u102))
(define-constant ERR-CANNOT-KICK-SELF (err u103))
(define-constant ERR-INVALID-SIGNATURE (err u104))
(define-constant ERR-NOT-JOINED (err u105))
(define-constant ERR-MESSAGE-HASH-FAILED (err u106))
(define-constant ERR-ALREADY-CLAIMED (err u107))
(define-constant ERR-DEPLOYER-NOT-LAST (err u108))

;; data vars
;;

(define-data-var total-players uint u0)

;; data maps
;;

(define-map players principal uint)
(define-map claimed-rewards principal bool)

;; public functions
;;

;; Join the lobby (deployer funds pool, others join free)
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

		;; If deployer is joining, transfer pool size to contract
		(if (is-eq sender DEPLOYER)
			(try! (contract-call? .STACKS-WARS-TOKEN transfer POOL-SIZE sender (as-contract tx-sender) none))
			true
		)

		;; Update state
		(map-insert players sender stacks-block-height)
		(var-set total-players (+ player-count u1))

		(ok true)
	)
)

;; Leave the lobby and withdraw deposit (only deployer gets refund)
;; @param signature: signature from stacks wars
;; @returns (ok true) on success
(define-public (leave (signature (buff 65)))
	(let
		(
			(sender tx-sender)
			(player-count (var-get total-players))
			(amount (if (is-eq sender DEPLOYER) POOL-SIZE u0))
			(message-hash (try! (construct-message-hash amount sender)))
		)
		;; Check if player has joined
		(asserts! (is-some (map-get? players sender)) ERR-NOT-JOINED)

		;; If deployer is leaving, ensure they are the last player
		(asserts!
			(or (not (is-eq sender DEPLOYER)) (is-eq player-count u1))
			ERR-DEPLOYER-NOT-LAST
		)

		;; Verify signature from stacks wars
		(asserts!
			(secp256k1-verify message-hash signature TRUSTED-PUBLIC-KEY)
			ERR-INVALID-SIGNATURE
		)

		;; If deployer is leaving, transfer pool size back
		(if (is-eq sender DEPLOYER)
			(try! (as-contract (contract-call? .STACKS-WARS-TOKEN transfer POOL-SIZE tx-sender sender none)))
			true
		)

		;; Update state
		(map-delete players sender)
		(var-set total-players (- player-count u1))

		(ok true)
	)
)

;; Claim after game completion
;; @param amount: reward amount in microSTX
;; @param signature: signature from stacks wars
;; @returns (ok true) on success
(define-public (claim (amount uint) (signature (buff 65)))
	(let
		(
			(sender tx-sender)
			(message-hash (try! (construct-message-hash amount sender)))
			(fee-amount (/ (* amount FEE-PERCENTAGE) u100))
			(reward-amount (- amount fee-amount))
		)
		;; Check if player has joined
		(asserts! (is-some (map-get? players sender)) ERR-NOT-JOINED)

		;; Check if player has already claimed
		(asserts! (is-none (map-get? claimed-rewards sender)) ERR-ALREADY-CLAIMED)

		;; Verify signature from stacks wars
		(asserts!
			(secp256k1-verify message-hash signature TRUSTED-PUBLIC-KEY)
			ERR-INVALID-SIGNATURE
		)

		;; Transfer fee to fee wallet
		(try! (as-contract (contract-call? .STACKS-WARS-TOKEN transfer fee-amount tx-sender FEE-WALLET none)))

		;; Transfer reward to player
		(try! (as-contract (contract-call? .STACKS-WARS-TOKEN transfer reward-amount tx-sender sender none)))

		;; Mark as claimed
		(map-set claimed-rewards sender true)

		(ok true)
	)
)

;; Kick a player from the lobby (creator only, before game starts)
;; @param player: principal address of player to kick
;; @param signature: signature from stacks wars
;; @returns (ok true) on success
(define-public (kick (player principal) (signature (buff 65)))
	(let
		(
			(sender tx-sender)
			(player-count (var-get total-players))
			(message-hash (try! (construct-message-hash u0 player)))
		)
		;; Only deployer can kick
		(asserts! (is-eq sender DEPLOYER) ERR-NOT-DEPLOYER)

		;; Check if player is in the lobby
		(asserts! (is-some (map-get? players player)) (ok false))

		;; Deployer cannot kick themselves
		(asserts! (not (is-eq player DEPLOYER)) ERR-CANNOT-KICK-SELF)

		;; Verify signature from stacks wars
		(asserts!
			(secp256k1-verify message-hash signature TRUSTED-PUBLIC-KEY)
			ERR-INVALID-SIGNATURE
		)

		;; Update state (no refund since players didn't pay)
		(map-delete players player)
		(var-set total-players (- player-count u1))

		(ok true)
	)
)

;; read only functions
;;

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

;; Construct message hash for signature verification
;; @param amount: amount to include in the message
;; @param player: principal address of the player
;; @returns (ok buff) containing the message hash
(define-private (construct-message-hash (amount uint) (player principal))
	(let ((message {
		amount: amount,
		player: player,
		contract: (as-contract tx-sender)
		}))
		(match (to-consensus-buff? message)
			buff (ok (sha256 buff))
			ERR-MESSAGE-HASH-FAILED
		)
	)
)

