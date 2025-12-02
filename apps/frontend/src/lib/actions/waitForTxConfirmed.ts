import { connectWebSocketClient } from "@stacks/blockchain-api-client";

const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

export const waitForTxConfirmed = async (txId: string): Promise<void> => {
	console.log("waiting for tx to confirm → starting");
	const controller = new AbortController();

	let resolvePromise: () => void;
	let rejectPromise: (reason?: unknown) => void;
	let wsUnsub: () => void = () => {};

	const combined = new Promise<void>((resolve, reject) => {
		resolvePromise = resolve;
		rejectPromise = reject;
	});

	// Helper function to check for specific error codes
	const isExpectedError = (reason: string | undefined): boolean => {
		if (!reason) return false;

		// More precise error code checking
		// Look for exact matches of error codes, accounting for different formats
		const errorPatterns = [
			/\bu5\b/i, // ERR_ALREADY_JOINED - word boundary ensures exact match
			/\bu9\b/i, // ERR_REWARD_ALREADY_CLAIMED - case insensitive
			/\(err u5\)/i, // Format: (err u5)
			/\(err u9\)/i, // Format: (err u9)
			/err-u5/i, // Format: err-u5
			/err-u9/i, // Format: err-u9
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

				if (isExpectedError(reason)) {
					console.log(
						"✅ Expected error (already joined/claimed), treating as success"
					);
					resolvePromise(); // Treat as success
				} else {
					console.error("❌ Unexpected transaction failure");
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

				if (isExpectedError(reason)) {
					console.log(
						"✅ WebSocket: Expected error (already joined/claimed), treating as success"
					);
					resolvePromise(); // Don't reject, allow flow to pass
				} else {
					console.error(
						"❌ WebSocket: Unexpected transaction failure"
					);
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
