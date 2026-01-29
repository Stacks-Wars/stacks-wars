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
		<>
			<Header />
			{children}
			{modal}
			<Footer />
		</>
	);
}
