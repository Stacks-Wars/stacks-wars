import Footer from "@/components/main/footer";
import Header from "@/components/main/header";

export default function GameLayout({
	children,
}: {
	children: React.ReactNode;
}) {
	return (
		<div className="bg-background text-foreground flex min-h-screen flex-col justify-between">
			<Header />
			{children}
			<Footer />
		</div>
	);
}
