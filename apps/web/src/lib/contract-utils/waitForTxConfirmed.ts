import { connectWebSocketClient } from "@stacks/blockchain-api-client";

/**
 * Expected error codes that should be treated as success
 */
export enum ExpectedError {
	ERR_ALREADY_JOINED = "u100",
	ERR_NOT_JOINED = "u105",
	ERR_ALREADY_CLAIMED = "u107",
}

/**
 * Wait for transaction to be confirmed
 */
export const waitForTxConfirmed = async (
	txId: string,
	expect?: ExpectedError
): Promise<void> => {
	console.log("waiting for tx to confirm → starting");
	const controller = new AbortController();

	const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

	let resolvePromise: () => void;
	let rejectPromise: (reason?: unknown) => void;
	let wsUnsub: () => void = () => {};

	const combined = new Promise<void>((resolve, reject) => {
		resolvePromise = resolve;
		rejectPromise = reject;
	});

	// Helper function to check for specific error codes
	const isExpectedError = (
		reason: string | undefined,
		expected?: ExpectedError
	): boolean => {
		if (!reason) return false;

		// If an expected error is specified, check if it matches
		if (expected) {
			return (
				reason.includes(`(err ${expected})`) ||
				reason.includes(`u${expected.slice(1)}`)
			);
		}

		const errorPatterns = [
			/\bu1\b/i, // insufficient funds
			/\bu2\b/i, // same principal
			/\bu3\b/i, // non-positive amount
			/\bu4\b/i, // sender not tx-sender
			/\bu100\b/i, // ERR_ALREADY_JOINED
			/\bu101\b/i, // ERR_DEPLOYER_MUST_JOIN_FIRST
			/\bu102\b/i, // ERR_NOT_DEPLOYER
			/\bu103\b/i, // ERR_CANNOT_KICK_SELF
			/\bu104\b/i, // ERR_INVALID_SIGNATURE
			/\bu105\b/i, // ERR_NOT_JOINED
			/\bu106\b/i, // ERR_MESSAGE_HASH_FAILED
			/\bu107\b/i, // ERR_ALREADY_CLAIMED
			/\bu108\b/i, // ERR_DEPLOYER_NOT_LAST
			/\(err u1\)/i,
			/\(err u2\)/i,
			/\(err u3\)/i,
			/\(err u4\)/i,
			/\(err u100\)/i,
			/\(err u101\)/i,
			/\(err u102\)/i,
			/\(err u103\)/i,
			/\(err u104\)/i,
			/\(err u105\)/i,
			/\(err u106\)/i,
			/\(err u107\)/i,
			/\(err u108\)/i,
		];

		return errorPatterns.some((pattern) => pattern.test(reason));
	};

	const pollTx = async () => {
		const check = async () => {
			const res = await fetch(
				`https://api.${network}.hiro.so/extended/v1/tx/${txId}`,
				{ signal: controller.signal }
			);
			if (!res.ok) throw new Error("Failed to fetch tx status");

			const data = await res.json();
			console.log("🔁 polling tx status:", data.tx_status);

			if (data.tx_status === "success") {
				resolvePromise();
				controller.abort();
				wsUnsub();
				return true;
			}

			if (
				data.tx_status === "abort_by_response" ||
				data.tx_status === "abort_by_post_condition"
			) {
				const reason = data.tx_result?.repr;
				console.log("📋 Transaction aborted, reason:", reason);

				// Check if this is an expected error that should be treated as success
				if (expect && isExpectedError(reason, expect)) {
					console.log(
						"📋 Expected error treated as success:",
						reason
					);
					resolvePromise();
					controller.abort();
					wsUnsub();
					return true;
				}

				if (isExpectedError(reason)) {
					rejectPromise(
						new Error(
							`Transaction failed: ${reason || "unknown reason"}`
						)
					);
				} else {
					rejectPromise(
						new Error(
							`Transaction failed or was aborted: ${reason || "unknown reason"}`
						)
					);
				}

				controller.abort();
				wsUnsub();
				return true;
			}

			return false;
		};

		const done = await check();
		if (done) return;

		// ⏱ Poll every 30 seconds
		const interval = setInterval(async () => {
			try {
				const done = await check();
				if (done) clearInterval(interval);
			} catch (err) {
				clearInterval(interval);
				rejectPromise(err);
			}
		}, 30_000);
	};

	const listenWebSocket = async () => {
		const client = await connectWebSocketClient(
			`wss://api.${network}.hiro.so/`
		);
		const sub = await client.subscribeTxUpdates(txId, (event) => {
			console.log("📡 [Tx Event]", event);

			if (event.tx_status === "success") {
				sub.unsubscribe();
				controller.abort();
				resolvePromise();
			}

			if (
				event.tx_status === "abort_by_response" ||
				event.tx_status === "abort_by_post_condition"
			) {
				const reason = event.tx_result?.repr;
				console.log(
					"📋 WebSocket: Transaction aborted, reason:",
					reason
				);

				// Check if this is an expected error that should be treated as success
				if (expect && isExpectedError(reason, expect)) {
					console.log(
						"📡 WebSocket: Expected error treated as success:",
						reason
					);
					sub.unsubscribe();
					controller.abort();
					resolvePromise();
					return;
				}

				if (isExpectedError(reason)) {
					rejectPromise(
						new Error(
							`Transaction failed: ${reason || "unknown reason"}`
						)
					);
				} else {
					rejectPromise(
						new Error(
							`Transaction failed or was aborted: ${reason || "unknown reason"}`
						)
					);
				}

				sub.unsubscribe();
				controller.abort();
			}
		});

		wsUnsub = () => sub.unsubscribe();
	};

	// Run both in parallel
	await Promise.allSettled([listenWebSocket(), pollTx()]);
	return combined.finally(() => {
		controller.abort();
		wsUnsub();
	});
};
