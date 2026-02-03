import Image from "next/image";

export default function Gameplay() {
	return (
		<div className="w-full relative shadow-[0px_0px_34.07px_-18.09px_rgba(255,_255,_255,_0.25)_inset] rounded-[22px] [background:linear-gradient(150.88deg,_#2c61b8,_#000_65.11%)] border-darkslategray border-solid border-[0.5px] box-border overflow-hidden text-center text-[19.6px] text-white font-neue-montreal">
			<div className="mt-4 mx-3 rounded-[7.82px] bg-gray border-darkslategray border-solid border-[0.5px] box-border flex items-center justify-center py-4 px-4 shrink-0">
				<b className="max-w-[240.5px] relative leading-[23.46px] inline-block shrink-0">
					<span>Engage in intense gameplay</span>
					<span className="text-xl">and</span>
					<span> battle other warriors for dominance.</span>
				</b>
			</div>
			<Image
				className="absolute bottom-0 -right-[10.64px] object-cover shrink-0"
				src={"/images/xbox-pad.png"}
				width={369.1}
				height={364.1}
				alt=""
			/>
			<Image
				className="absolute top-[116.83px] left-[261.52px] w-[25.9px] h-[30.9px] object-cover shrink-0"
				width={25.9}
				src={"/images/ruby1.png"}
				height={30.9}
				sizes="100vw"
				alt=""
			/>
			<Image
				className="absolute top-[262.99px] left-[31.33px] w-[25.9px] h-[30.9px] object-contain shrink-0"
				src={"/images/ruby2.png"}
				width={25.9}
				height={30.9}
				sizes="100vw"
				alt=""
			/>
		</div>
	);
}
