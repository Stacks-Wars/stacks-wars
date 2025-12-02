"use client";
import Loading from "@/app/(games)/lexi-wars/[lobbyId]/loading";
import RequireAuth from "@/components/require-auth";
import { useChatSocket, UseChatSocketType } from "@/hooks/useChatSocket";
import { getClaimFromJwt } from "@/lib/getClaimFromJwt";
import { useParams } from "next/navigation";
import { createContext, useContext, useEffect, useState } from "react";

const ChatSocketContext = createContext<UseChatSocketType | undefined>(
	undefined
);

interface ChatProviderProps {
	children: Readonly<React.ReactNode>;
}

export const ChatSocketProvider = ({ children }: ChatProviderProps) => {
	const params = useParams();
	const [userId, setUserId] = useState<string | null>(null);
	const [isLoading, setIsLoading] = useState<boolean>(true);
	const lobbyId = typeof params.lobbyId === "string" ? params.lobbyId : "";

	const socketData = useChatSocket({ lobbyId, userId: userId ?? "" });

	useEffect(() => {
		const fetchUserId = async () => {
			try {
				setIsLoading(true);
				const id = await getClaimFromJwt<string>("sub");
				setUserId(id);
			} catch (error) {
				console.error("Failed to get user ID from JWT:", error);
				setUserId(null);
			} finally {
				setIsLoading(false);
			}
		};

		fetchUserId();
	}, []);

	if (isLoading) {
		return <Loading />;
	}

	if (!userId || !lobbyId) {
		return <RequireAuth />;
	}

	return (
		<ChatSocketContext.Provider value={socketData}>
			{children}
		</ChatSocketContext.Provider>
	);
};

export const useChatSocketContext = () => {
	const context = useContext(ChatSocketContext);
	if (!context) {
		throw new Error(
			"useChatSocketContext must be used within a ChatSocketProvider"
		);
	}
	return context;
};
