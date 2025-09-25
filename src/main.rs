mod emulator;
use crate::emulator::FlewittPCEmulator;


fn main() {
    const BIN_PATH: &str = "./workspace/output.bin";
    let mut emulator = FlewittPCEmulator::new_from_binary_file(BIN_PATH);
    emulator.run();

}
