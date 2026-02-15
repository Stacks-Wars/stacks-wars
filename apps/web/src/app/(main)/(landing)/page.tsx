import Image from "next/image";
import { Suspense } from "react";
import GamesShowcase, {
	GamesShowcaseSkeleton,
} from "./_components/games-showcase";
import Hero from "./_components/hero";
import PlayToEarn from "./_components/play-to-earn";
import TryOut from "./_components/try-out";

export default function HomePage() {
	return (
		<div className="container mx-auto overflow-x-hidden px-4 py-8">
			<Hero />
			<div>
				<Suspense fallback={<GamesShowcaseSkeleton />}>
					<GamesShowcase />
				</Suspense>
				<Image
					src={"/images/footer-seperator.svg"}
					alt="Footer Illustration"
					width={1248}
					height={28}
					className="mb-6 h-4 w-full object-cover sm:mb-8 sm:h-7 lg:mb-12 mt-6"
				/>
			</div>
			<PlayToEarn />
			<TryOut />
		</div>
	);
}
