import Header from "@/components/home/header";
import Footer from "@/components/home/footer";
import { ConnectUserProvider } from "@/contexts/ConnectWalletContext";

export default function Layout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<ConnectUserProvider>
			<Header />
			<main className="flex-1 bg-primary/10">{children}</main>
			<Footer />
		</ConnectUserProvider>
	);
}
