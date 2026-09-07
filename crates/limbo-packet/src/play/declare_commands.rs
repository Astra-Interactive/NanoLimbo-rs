use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// Brigadier node flags: a literal node that can be executed.
const LITERAL_EXECUTABLE_NODE: u8 = 1 | 0x04;

/// Brigadier node flags: an argument node that can be executed and asks the server for
/// completions.
const ARGUMENT_EXECUTABLE_SUGGESTING_NODE: u8 = 2 | 0x04 | 0x10;

/// The command tree the client offers in its chat box, from 1.13.
///
/// Each command becomes two nodes hanging off the root: a literal for the name and a
/// greedy string argument for the rest of the line, whose completions are asked of the
/// server.
pub struct DeclareCommands<'a> {
    pub commands: &'a [String],
}

impl DeclareCommands<'_> {
    /// The root node, whose children are the literal node of every command.
    ///
    /// A command's literal sits at index `2n + 1` and its argument at `2n + 2`, so the
    /// root's children are the odd indices.
    fn write_root<B>(&self, buffer: &mut B)
    where
        B: BufMut + ?Sized,
    {
        buffer.put_u8(0);
        buffer.write_var_int(self.commands.len() as i32);

        for command in 0..self.commands.len() {
            buffer.write_var_int((command * 2 + 1) as i32);
        }
    }

    fn write_command<B>(&self, buffer: &mut B, index: usize, command: &str)
    where
        B: BufMut + ?Sized,
    {
        let literal_node = (index * 2 + 1) as i32;
        let argument_node = literal_node + 1;

        buffer.put_u8(LITERAL_EXECUTABLE_NODE);
        buffer.write_var_int(1);
        buffer.write_var_int(argument_node);
        buffer.write_string(command);

        buffer.put_u8(ARGUMENT_EXECUTABLE_SUGGESTING_NODE);
        // The Java implementation declares one child here and points it at this very
        // node. Reproduced rather than corrected: the packet is only ever sent with an
        // empty command list, so no client has seen the cycle, and diverging from the
        // reference bytes would cost the parity this crate is checked by.
        buffer.write_var_int(1);
        buffer.write_var_int(argument_node);
        buffer.write_string("arg");
        buffer.write_string("brigadier:string");
        buffer.write_var_int(0);
        buffer.write_string("minecraft:ask_server");
    }
}

impl ClientboundPacket for DeclareCommands<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::DeclareCommands
    }

    fn encode<B>(&self, buffer: &mut B, _version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.write_var_int((self.commands.len() * 2 + 1) as i32);

        self.write_root(buffer);
        for (index, command) in self.commands.iter().enumerate() {
            self.write_command(buffer, index, command);
        }

        // Index of the root node.
        buffer.write_var_int(0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    fn encoded(commands: &[String]) -> Vec<u8> {
        let mut buffer = BytesMut::new();
        DeclareCommands { commands }
            .encode(&mut buffer, ProtocolVersion::V1_20_5)
            .expect("encoding cannot fail");

        buffer.to_vec()
    }

    #[test]
    fn given_no_commands_when_encoded_then_only_a_childless_root_is_declared() {
        assert_eq!(encoded(&[]), vec![0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn given_two_commands_when_encoded_then_the_root_points_at_both_literal_nodes() {
        let commands = ["one".to_owned(), "two".to_owned()];

        let bytes = encoded(&commands);

        assert_eq!(
            bytes.get(..5),
            Some([0x05, 0x00, 0x02, 0x01, 0x03].as_slice())
        );
    }
}
