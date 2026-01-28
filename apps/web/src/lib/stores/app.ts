/**
 * App Store
 *
 * Persisted store for app-level preferences and settings.
 * This includes lobby filters, pagination offsets, and other user preferences.
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { LobbyStatus } from "@/lib/definitions";
import type { CreateLobbyRequest, Lobby } from "@/lib/definitions";
import { ApiClient, type ApiResponse } from "@/lib/api/client";
import { waitForTxConfirmed } from "@/lib/contract-utils/waitForTxConfirmed";
import {
	joinNormalContract,
	joinSponsoredContract,
} from "@/lib/contract-utils/join";
import { toast } from "sonner";
import type { ContractIdString, AssetString } from "@stacks/transactions";

interface LobbyCreationProgress {
	contractAddress: string;
	step: "deployed" | "joined";
	payload: CreateLobbyRequest;
	restoredFromStorage?: boolean;
}

interface AppActions {
	setLobbyFilter: (filter: LobbyStatus[]) => void;
	setLobbyOffset: (offset: number) => void;
	setLobbyCreationProgress: (progress: LobbyCreationProgress) => void;
	clearLobbyCreationProgress: () => void;
	handleContinue: (userWalletAddress: string) => Promise<ApiResponse<Lobby>>;
}

interface AppStore {
	lobbyFilter: LobbyStatus[];
	lobbyOffset: number;
	hasHydrated: boolean;
	lobbyCreationProgress: LobbyCreationProgress | null;

	actions: AppActions;
}

const useAppStore = create<AppStore>()(
	persist(
		(set, get) => ({
			lobbyFilter: ["waiting", "inProgress"],
			lobbyOffset: 0,
			hasHydrated: false,
			lobbyCreationProgress: null,

			actions: {
				setLobbyFilter: (filter) => set({ lobbyFilter: filter }),
				setLobbyOffset: (offset) => set({ lobbyOffset: offset }),
				setLobbyCreationProgress: (progress) => {
					set({
						lobbyCreationProgress: {
							...progress,
							contractAddress: progress.contractAddress,
							step: progress.step,
							payload: progress.payload,
							restoredFromStorage: false,
						},
					});
				},
				clearLobbyCreationProgress: () =>
					set({ lobbyCreationProgress: null }),
				handleContinue: async (userWalletAddress: string) => {
					const progress = get().lobbyCreationProgress;
					if (!progress) {
						return {
							status: 400,
							error: "No lobby creation progress found",
						};
					}

					try {
						if (progress.step === "deployed") {
							// Continue by joining the contract
							const isSponsored = progress.payload?.isSponsored;
							const txId = isSponsored
								? await joinSponsoredContract({
										contract:
											progress.contractAddress as ContractIdString,
										amount:
											progress.payload.currentAmount || 0,
										isCreator: true,
										tokenId:
											`${progress.payload.tokenContractId}::${progress.payload.tokenSymbol}` as AssetString,
										address: userWalletAddress,
									})
								: await joinNormalContract({
										contract:
											progress.contractAddress as ContractIdString,
										amount:
											progress.payload.entryAmount || 0,
										address: userWalletAddress,
									});

							if (!txId) {
								toast.error("Failed to join contract", {
									description: "Please try again later.",
								});
								return {
									status: 500,
									error: "Failed to join contract: No transaction ID returned",
								};
							}

							await waitForTxConfirmed(txId);
							set({
								lobbyCreationProgress: {
									...progress,
									step: "joined",
								},
							});
							toast.success("Successfully joined the contract!");
						}
						// Post the lobby
						const response = await ApiClient.post<Lobby>(
							"/api/lobby",
							progress.payload
						);
						return response;
					} catch (err) {
						toast.error("Failed to continue lobby creation");
						console.error(err);
						return {
							status: 500,
							error: "Failed to continue lobby creation",
						};
					}
				},
			},
		}),
		{
			name: "app-storage",
			partialize: (state) => ({
				lobbyFilter: state.lobbyFilter,
				lobbyOffset: state.lobbyOffset,
				lobbyCreationProgress: state.lobbyCreationProgress,
			}),
			onRehydrateStorage: () => (state, error) => {
				if (state && state.lobbyCreationProgress) {
					// Mark progress as restored from storage
					state.lobbyCreationProgress.restoredFromStorage = true;
				}
				if (state) {
					state.hasHydrated = true;
				}
			},
		}
	)
);

export const useLobbyFilter = () => useAppStore((state) => state.lobbyFilter);
export const useLobbyOffset = () => useAppStore((state) => state.lobbyOffset);
export const useAppHasHydrated = () =>
	useAppStore((state) => state.hasHydrated);
export const useLobbyCreationProgress = () =>
	useAppStore((state) => state.lobbyCreationProgress);
export const useAppActions = () => useAppStore((state) => state.actions);
