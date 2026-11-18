// Keep this invalid operand isolated: its eager diagnostic contaminates other
// selected expressions if placed beside the successful probes.
a: 3 & (_ + 1)
