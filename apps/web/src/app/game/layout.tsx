import Header from "@/components/main/header";

export default function GameLayout({
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
		</>
	);
}
