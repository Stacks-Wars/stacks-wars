import Footer from "@/components/main/footer";
import Header from "@/components/main/header";

export default function GameLayout({
	children,
}: {
	children: React.ReactNode;
}) {
	return (
		<div className="flex min-h-screen flex-col justify-between">
			<Header />
			<main className="container mx-auto px-4 py-8">{children}</main>
			<Footer />
		</div>
	);
}
