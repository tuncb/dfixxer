unit LeadingExpressionIndent;

interface

type
  TVec = record
    X, Y, Z: Double;
  end;

implementation

function Det4(const P1, P2, P3, P4: TVec): Double;
begin
  Result :=
      P1.X * (
          P2.Y * (P3.Z - P4.Z)
        - P3.Y * (P2.Z - P4.Z)
        + P4.Y * (P2.Z - P3.Z) )
    - P2.X * (
          P1.Y * (P3.Z - P4.Z)
        - P3.Y * (P1.Z - P4.Z)
        + P4.Y * (P1.Z - P3.Z) )
    + P3.X * (
          P1.Y * (P2.Z - P4.Z)
        - P2.Y * (P1.Z - P4.Z)
        + P4.Y * (P1.Z - P2.Z) )
    - P4.X * (
          P1.Y * (P2.Z - P3.Z)
        - P2.Y * (P1.Z - P3.Z)
        + P3.Y * (P1.Z - P2.Z) );
end;

end.