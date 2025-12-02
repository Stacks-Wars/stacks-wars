"use client";
import {
	createContext,
	useContext,
	useEffect,
	useState,
	ReactNode,
} from "react";
import { apiRequest } from "@/lib/api";
import { getClaimFromJwt } from "@/lib/getClaimFromJwt";
import { User } from "@/types/schema/user";

interface UserContextType {
	user: User | null;
	loading: boolean;
	refetchUser: () => Promise<void>;
}

const UserContext = createContext<UserContextType | undefined>(undefined);

export const UserProvider = ({ children }: { children: ReactNode }) => {
	const [user, setUser] = useState<User | null>(null);
	const [loading, setLoading] = useState(false);

	const fetchUser = async () => {
		try {
			setLoading(true);
			const userId = await getClaimFromJwt<string>("sub");
			if (!userId) {
				setUser(null);
				return;
			}

			const userData = await apiRequest<User>({
				path: `/user/${userId}`,
				method: "GET",
			});
			setUser(userData);
		} catch (error) {
			console.error("Failed to fetch user:", error);
			setUser(null);
		} finally {
			setLoading(false);
		}
	};

	const refetchUser = async () => {
		await fetchUser();
	};

	useEffect(() => {
		fetchUser();
	}, []);

	return (
		<UserContext.Provider value={{ user, loading, refetchUser }}>
			{children}
		</UserContext.Provider>
	);
};

export const useUser = () => {
	const context = useContext(UserContext);
	if (!context) {
		throw new Error("useUser must be used within a UserProvider");
	}
	return context;
};
