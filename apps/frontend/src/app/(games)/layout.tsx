import Chat from "@/components/games/chat";
import { ChatSocketProvider } from "@/contexts/ChatSocketProvider";
import { ConnectUserProvider } from "@/contexts/ConnectWalletContext";

interface LayoutProps {
	children: Readonly<React.ReactNode>;
}

export default function Layout({ children }: LayoutProps) {
	return (
		<ConnectUserProvider>
			<main className="bg-primary/10 flex-1">
				<ChatSocketProvider>
					{children}
					<Chat />
				</ChatSocketProvider>
			</main>
		</ConnectUserProvider>
	);
}
