import Header from "@/components/main/header";
import Footer from "@/components/main/footer";

export default function MainLayout({
	children,
	modal,
}: {
	children: React.ReactNode;
	modal: React.ReactNode;
}) {
	return (
		<div className="bg-app-spotlight text-foreground flex min-h-screen flex-col justify-between bg-fixed">
			<Header />
			{children}
			{modal}
			<Footer />
		</div>
	);
}
