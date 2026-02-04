unit GenericExpressionSpacing;

interface

type
  TMy = class
  public
    procedure Foo();
  end;

implementation

procedure TMy.Foo();
var
  LObjects: TList<PObject>;
begin
  LObjects := TList<PObject>.Create();
  LObjects := TDictionary<String, TList<Integer>>.Create();
end;

end.
