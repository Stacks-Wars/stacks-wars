import Community from "./_components/community";
import Description from "./_components/description";
import HeroSection from "./_components/hero-section";
import Roadmap from "./_components/roadmap";

export default function LandingPage() {
	return (
		<div className="snap-y snap-mandatory overflow-y-auto ">
			<HeroSection />

			<Description />

			<Roadmap />

			<Community />
		</div>
	);
}
