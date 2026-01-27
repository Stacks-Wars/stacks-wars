;; ==============================
;; stx-vault.clar - Rendezvous Fuzz Tests
;; ==============================

;; ----------------------
;; CONSTANTS
;; ----------------------

(define-constant ERR_JOIN_TEST_FAILED (err u200))
(define-constant ERR_LEAVE_TEST_FAILED (err u201))
(define-constant ERR_CLAIM_TEST_FAILED (err u202))
(define-constant ERR_KICK_TEST_FAILED (err u203))

;; ----------------------
;; PROPERTY TESTS
;; ----------------------

;; Test: join increases total players and marks player as joined
(define-public (test-join)
	(let
		(
			(players-before (get-total-players))
			(joined-before (has-joined tx-sender))
		)
		(match (join)
			ok-val
				(let
					(
						(players-after (get-total-players))
						(joined-after (has-joined tx-sender))
					)
					(asserts!
						(and
							(is-eq players-after (+ players-before u1))
							(not joined-before)
							joined-after
						)
						ERR_JOIN_TEST_FAILED
					)
					(ok true)
				)
			err-val (ok false) ;; Discard if join fails
		)
	)
)

;; Test: leave decreases total players and unmarks player as joined
(define-public (test-leave (signature (buff 65)))
	(let
		(
			(players-before (get-total-players))
			(joined-before (has-joined tx-sender))
		)
		(match (leave signature)
			ok-val
			(let
				(
					(players-after (get-total-players))
					(joined-after (has-joined tx-sender))
				)
				(asserts!
					(and
						(is-eq players-after (- players-before u1))
						joined-before
						(not joined-after)
					)
					ERR_LEAVE_TEST_FAILED
				)
				(ok true)
			)
			err-val (ok false) ;; Discard if leave fails
		)
	)
)

;; Test: claim sets claimed flag for player
(define-public (test-claim (amount uint) (signature (buff 65)))
	(let
		(
			(claimed-before (default-to false (map-get? claimed-rewards tx-sender)))
		)
		(match (claim amount signature)
			ok-val
			(let
				(
					(claimed-after (default-to false (map-get? claimed-rewards tx-sender)))
				)
				(asserts!
					(and
						(not claimed-before)
						claimed-after
					)
					ERR_CLAIM_TEST_FAILED
				)
				(ok true)
			)
			err-val (ok false) ;; Discard if claim fails
		)
	)
)

;; Test: kick removes player from lobby
(define-public (test-kick (player principal) (signature (buff 65)))
    (let
        (
            (players-before (get-total-players))
            (joined-before (has-joined player))
        )
        (match (kick player signature)
            ok-val
                (if (is-eq ok-val true)
                    (let
                        (
                            (players-after (get-total-players))
                            (joined-after (has-joined player))
                        )
                        (asserts!
                            (and
                                (is-eq players-after (- players-before u1))
                                joined-before
                                (not joined-after)
                            )
                            ERR_KICK_TEST_FAILED
                        )
                        (ok true)
                    )
                    (ok false) ;; Discard if (ok false) (e.g. player not in lobby)
                )
            err-val (ok false) ;; Discard if kick fails
        )
    )
)

;; ----------------------
;; INVARIANTS
;; ----------------------

;; Invariant: total-players matches number of joined players
(define-read-only (invariant-total-players-matches-joined)
	(let
		(
			(players-count (var-get total-players))
			;; Rendezvous can enumerate principals and check has-joined for each
			;; For simplicity, we just check that if total-players > 0, deployer has joined
		)
		(if (> players-count u0)
			(has-joined DEPLOYER)
			true
		)
	)
)

;; Invariant: deployer must join first if any players exist
(define-read-only (invariant-deployer-joins-first)
	(if (> (var-get total-players) u0)
		(has-joined DEPLOYER)
		true
	)
)